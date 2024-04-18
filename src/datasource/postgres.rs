use crate::{expr, query::QueryType, Expression, Query, Renderable};
use async_trait::async_trait;
use postgres::NoTls;
use tokio_postgres::{Client, GenericClient};

use crate::traits::datasource::DataSource;

// #[async_trait]
// impl DataSource for Client {
//     fn query_fetch(&self, query: &Query) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
//         let query = query.render();
//         let rows = self.query(&query, &[]).await?;

//         let mut result = Vec::new();
//         for row in rows {
//             let mut result_row = Vec::new();
//             for col in row.columns() {
//                 let value: String = row.get(col.name());
//                 result_row.push(value);
//             }
//             result.push(result_row);
//         }

//         Ok(result)
//     }

//     async fn query_exec(&self, query: &crate::Query) -> Result<(), Box<dyn std::error::Error>> {
//         todo!()
//     }
// }

type Cell = dyn tokio_postgres::types::ToSql + Sync + 'static;
type Rows<'a> = Vec<&'a Cell>;

trait InsertRows<'a> {
    async fn insert_rows(
        &self,
        query: Query<'a>,
        rows: Box<dyn Iterator<Item = Rows<'a>>>,
    ) -> Result<Vec<i32>, Box<dyn std::error::Error>>;
}

impl<'a> InsertRows<'a> for Client {
    async fn insert_rows(
        &self,
        query: Query<'a>,
        rows: Box<dyn Iterator<Item = Rows<'a>>>,
    ) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
        let statement = self.prepare(&query.render()).await?;

        let mut ids = Vec::new();
        for row_set in rows {
            let row = self.query_one(&statement, &row_set).await?;
            ids.push(row.get(0));
        }

        Ok(ids)
    }
}

#[tokio::test]
async fn test_insert_async() {
    let (client, connection) = tokio_postgres::connect("host=localhost dbname=postgres", NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let query = Query::new("client")
        .set_type(QueryType::Insert)
        .add_column_field("name")
        .add_column_field("email")
        .add_column_field("is_vip");

    let rows: Vec<Rows> = vec![
        vec![&"John", &"john@gmail.com", &true as &Cell],
        vec![&"Jane", &"jane@gmail.com", &false as &Cell],
    ];

    let row_iter = Box::new(rows.into_iter());

    let ids = client.insert_rows(query, row_iter).await.unwrap();

    // should be sequential
    assert!(ids[0] + 1 == ids[1]);

    let id1 = ids[0].to_string();
    let id2 = ids[1].to_string();
    let expr = expr!("id in ({}, {})", &id1, &id2);

    let delete_query = Query::new("client")
        .set_type(QueryType::Delete)
        .add_condition(&expr);

    client.query(&delete_query.render(), &vec![]).await.unwrap();
}
