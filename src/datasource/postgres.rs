#![allow(dead_code)]

use crate::query::Query;
use crate::traits::datasource::DataSource;
use crate::traits::sql_chunk::SqlChunk;
use anyhow::Context;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::json;
use serde_json::Value;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use tokio_postgres::Row;

struct Postgres<'a> {
    client: &'a Client,
}

impl<'a> Postgres<'a> {
    pub fn new(client: &'a Client) -> Postgres<'a> {
        Postgres { client }
    }

    pub fn convert_value_tosql(&self, value: Value) -> Box<dyn ToSql + Sync> {
        match value {
            Value::Null => Box::new(None as Option<&[u8]>),
            Value::Bool(b) => Box::new(b),
            Value::Number(n) => {
                if n.is_i64() {
                    Box::new(n.as_i64().unwrap() as i32)
                } else {
                    Box::new(n.as_f64().unwrap() as i32)
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

    pub async fn query_raw(&self, query: &Query) -> Result<Vec<Value>> {
        let query_rendered = query.render_chunk();
        let params_tosql = query_rendered
            .params()
            .iter()
            .map(|v| self.convert_value_tosql(v.clone()))
            .collect::<Vec<_>>();

        let params_tosql_refs = params_tosql
            .iter()
            .map(|b| b.as_ref())
            .collect::<Vec<&(dyn ToSql + Sync)>>();

        let result = self
            .client
            .query(&query_rendered.sql_final(), params_tosql_refs.as_slice())
            .await
            .context(anyhow!("Error in query"))?;

        let mut results = Vec::new();
        for row in result {
            results.push(self.convert_value_fromsql(row)?);
        }

        Ok(results)
    }

    pub async fn query_one(&self, query: &Query) -> Result<Value> {
        let Some(res) = self.query_raw(query).await?.into_iter().next() else {
            return Err(anyhow!("No rows for query_one"));
        };
        Ok(res)
    }
    pub async fn query_opt(&self, query: &Query) -> Result<Option<Value>> {
        Ok(self.query_raw(query).await?.into_iter().next())
    }
}

trait InsertRows {
    async fn insert_rows(&self, query: &Query, rows: &Vec<Vec<Value>>) -> Result<Vec<Value>>;
}

impl<'a> InsertRows for Postgres<'a> {
    async fn insert_rows(&self, query: &Query, rows: &Vec<Vec<Value>>) -> Result<Vec<Value>> {
        // no rows to insert
        if rows.len() == 0 {
            return Ok(vec![]);
        }

        let query_rendered = query.render_chunk();
        let num_rows = query_rendered.params().len();

        if rows.len() == 0 {
            return Err(anyhow!("Insert query contains zero fields"));
        }

        let statement = self
            .client
            .prepare(&query_rendered.sql_final())
            .await
            .context("Attempting to execute an insert query")?;

        let mut row_cnt = 0;
        let mut ids = Vec::new();
        for row_set in rows {
            row_cnt += 1;
            if row_set.len() != num_rows {
                return Err(anyhow!(
                    "Number of columns in a row {} does not match number of fields in a query {} at row {}",
                    row_set.len(), num_rows, row_cnt
                ));
            }

            let params_tosql = row_set
                .iter()
                .map(|v| self.convert_value_tosql(v.clone()))
                .collect::<Vec<_>>();

            let params_tosql_refs = params_tosql
                .iter()
                .map(|b| b.as_ref())
                .collect::<Vec<&(dyn ToSql + Sync)>>();

            let row = self
                .client
                .query_one(&statement, params_tosql_refs.as_slice())
                .await?;

            let row = self.convert_value_fromsql(row)?;

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

trait SelectRows {
    async fn select_rows(&self, query: &Query) -> Result<Vec<Value>>;
}

impl<'a> SelectRows for Postgres<'a> {
    async fn select_rows(&self, query: &Query) -> Result<Vec<Value>> {
        // let (sql, params) = query.render_chunks();
        self.query_raw(query).await
    }
}

impl<'a> DataSource for Postgres<'a> {
    async fn query_fetch(&self, _query: &Query) -> Result<Vec<serde_json::Map<String, Value>>> {
        todo!()
    }

    async fn query_exec(&self, _query: &Query) -> Result<()> {
        todo!()
    }

    async fn query_insert(&self, _query: &Query, _rows: Vec<Vec<Value>>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::expression::Expression;
    use crate::query::QueryType;
    use crate::{expr, query::Query};
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

        let postgres = Postgres::new(&client);

        let query = Query::new("client")
            .set_type(QueryType::Insert)
            .add_column_field("name")
            .add_column_field("email")
            .add_column_field("is_vip");

        let rows: Vec<Vec<Value>> = vec![
            vec![json!("John"), json!("john@gamil.com"), json!(true)],
            vec![json!("Jane"), json!("jave@ffs.org"), json!(true)],
        ];

        dbg!(&query.render_chunk());
        let ids = postgres.insert_rows(&query, &rows).await.unwrap();

        // should be sequential
        assert!(ids[0].as_i64().unwrap() + 1 == ids[1].as_i64().unwrap());
        let id0 = ids[0].as_i64().unwrap() as i32;
        let id1 = ids[1].as_i64().unwrap() as i32;

        let expr = expr!("id in ({}, {})", id0, id1);

        let delete_query = Query::new("client")
            .set_type(QueryType::Delete)
            .add_condition(expr);

        postgres.query_raw(&delete_query).await.unwrap();
    }
}
