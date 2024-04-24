use crate::{Query, Renderable};
use anyhow::Context;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::json;
use serde_json::Value;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use tokio_postgres::Row;
use tokio_postgres::Statement;

struct Postgres<'a> {
    client: &'a Client,
}

impl Postgres<'_> {
    pub fn new(client: &Client) -> Postgres {
        Postgres { client }
    }

    pub fn convert_value_tosql(&self, value: Value) -> Box<dyn ToSql + Sync> {
        match value {
            Value::Null => Box::new(None as Option<&[u8]>),
            Value::Bool(b) => Box::new(b),
            Value::Number(n) => {
                if n.is_i64() {
                    Box::new(n.as_i64().unwrap())
                } else {
                    Box::new(n.as_f64().unwrap())
                }
            }
            Value::String(s) => Box::new(s),
            Value::Array(a) => Box::new(serde_json::to_string(&a).unwrap()),
            Value::Object(o) => Box::new(serde_json::to_string(&o).unwrap()),
        }
    }

    pub fn convert_value_fromsql(&self, row: Row) -> Result<Value> {
        let mut json_map: IndexMap<String, Value> = IndexMap::new();

        for (i, col) in row.columns().iter().enumerate() {
            let name = col.name().to_string();
            let col_type = col.type_().name();
            let value = match col_type {
                "int4" => json!(row.get::<_, Option<i32>>(i)), // int4 as i32
                "varchar" | "text" => json!(row.get::<_, Option<String>>(i)), // varchar and text as String
                "bool" => json!(row.get::<_, Option<bool>>(i)),               // bool as bool
                "float4" => json!(row.get::<_, Option<f32>>(i)),              // float4 as f32
                "float8" => json!(row.get::<_, Option<f64>>(i)),              // float8 as f64
                // "date" => row
                //     .get::<_, Option<chrono::NaiveDate>>(i)
                //     .map(|d| json!(d.to_string())), // date as ISO8601 string
                // "timestamp" => row
                //     .get::<_, Option<chrono::NaiveDateTime>>(i)
                //     .map(|dt| json!(dt.to_string())), // timestamp as ISO8601 string
                _ => {
                    return Err(anyhow!(
                        "Unsupported type: {} for column {}",
                        col_type,
                        name
                    ))
                }
            };

            json_map.insert(name, value);
        }

        Ok(json!(json_map))
    }

    pub async fn prepare(&self, query: &str) -> Result<Statement> {
        self.client
            .prepare(query)
            .await
            .context("Failed to prepare SQL query")
    }

    pub async fn query(&self, statement: &Statement, params: Value) -> Result<Vec<Value>> {
        if !params.is_array() {
            return Err(anyhow!("Params must be an array"));
        }

        let params_tosql = params
            .as_array()
            .unwrap()
            .iter()
            .map(|v| self.convert_value_tosql(v.clone()))
            .collect::<Vec<_>>();

        let params_tosql_refs = params_tosql
            .iter()
            .map(|b| b.as_ref())
            .collect::<Vec<&(dyn ToSql + Sync)>>();

        let result = self
            .client
            .query(statement, params_tosql_refs.as_slice())
            .await
            .context(anyhow!("Error in query"))?;

        let mut results = Vec::new();
        for row in result {
            results.push(self.convert_value_fromsql(row)?);
        }

        Ok(results)
    }

    pub async fn query_one(&self, statement: &Statement, params: Value) -> Result<Value> {
        if !params.is_array() {
            return Err(anyhow!("Params must be an array"));
        }

        let params_tosql = params
            .as_array()
            .unwrap()
            .iter()
            .map(|v| self.convert_value_tosql(v.clone()))
            .collect::<Vec<_>>();

        let params_tosql_refs = params_tosql
            .iter()
            .map(|b| b.as_ref())
            .collect::<Vec<&(dyn ToSql + Sync)>>();

        let result = self
            .client
            .query_one(statement, params_tosql_refs.as_slice())
            .await
            .context(anyhow!("Error in query"))?;

        self.convert_value_fromsql(result)
    }
}

type Cell = dyn tokio_postgres::types::ToSql + Sync + 'static;
type IRow<'a> = Vec<&'a Cell>;

trait InsertRows<'a> {
    async fn insert_rows(
        &self,
        query: Query<'a>,
        rows: Box<dyn Iterator<Item = Value>>,
    ) -> Result<Vec<Value>>;
}

impl<'a> InsertRows<'a> for Postgres<'a> {
    async fn insert_rows(
        &self,
        query: Query<'a>,
        rows: Box<dyn Iterator<Item = Value>>,
    ) -> Result<Vec<Value>> {
        let statement = self.prepare(&query.render()).await?;

        let mut ids = Vec::new();
        for row_set in rows {
            let row = self.query_one(&statement, row_set).await?;
            let row = if let Value::Object(obj) = row {
                obj
            } else {
                return Err(anyhow!("Expected query_one to return an Value::Object"));
            };

            let id = row
                .into_iter()
                .next()
                .context("query_one returned empty object")?
                .1;

            ids.push(id)
        }

        Ok(ids)
    }
}

trait SelectRows<'a> {
    async fn select_rows(&self, query: Query<'a>) -> Result<Vec<Value>>;
}

impl<'a> SelectRows<'a> for Postgres<'a> {
    async fn select_rows(&self, query: Query<'a>) -> Result<Vec<Value>> {
        // let (sql, params) = query.render_chunks();
        let statement = self.prepare(&query.render()).await?;
        self.query(&statement, query.param_values()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryType;
    use crate::Expression;
    use crate::{expr, Query, Renderable};
    use tokio_postgres::NoTls;

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

        let rows: Vec<IRow> = vec![
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
}
