#![allow(dead_code)]

use std::ops::Deref;
use std::sync::Arc;

use crate::dataset::ReadableDataSet;
use crate::prelude::EmptyEntity;
use crate::sql::chunk::Chunk;
use crate::sql::expression::{Expression, ExpressionArc};
use crate::sql::Query;
use crate::traits::datasource::DataSource;
use anyhow::Context;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use tokio_postgres::Row;

#[derive(Clone, Debug)]
pub struct Postgres {
    client: Arc<Box<Client>>,
}

/// Postgres is equal to its clones.
impl PartialEq for Postgres {
    fn eq(&self, other: &Postgres) -> bool {
        Arc::ptr_eq(&self.client, &other.client)
    }
}

impl Postgres {
    pub fn new(client: Arc<Box<Client>>) -> Postgres {
        Postgres { client }
    }

    pub fn escape(&self, expr: String) -> String {
        format!("\"{}\"", expr)
    }

    pub fn annotate_type(&self, expr: String, as_type: String) -> String {
        format!("{}::{}", expr, as_type)
    }

    pub fn convert_value_tosql(&self, value: Value) -> Box<dyn ToSql + Sync> {
        match value {
            Value::Null => Box::new(None as Option<&[u8]>),
            Value::Bool(b) => Box::new(b),
            Value::Number(n) => {
                if n.is_i64() {
                    Box::new(n.as_i64().unwrap() as i32)
                } else {
                    Box::new(n.as_f64().unwrap() as f32)
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
                "int8" => json!(row.get::<_, Option<i64>>(i)), // int8 as i64
                "varchar" | "text" => json!(row.get::<_, Option<String>>(i)), // varchar and text as String
                "bool" => json!(row.get::<_, Option<bool>>(i)),               // bool as bool
                "float4" => json!(row.get::<_, Option<f32>>(i)),              // float4 as f32
                "float8" => json!(row.get::<_, Option<f64>>(i)),              // float8 as f64
                "numeric" => json!(row.get::<_, Option<Decimal>>(i)),         // numeric as f64
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

    pub fn client(&self) -> &tokio_postgres::Client {
        self.client.as_ref()
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
            .context(anyhow!("Error in query {}", query.preview()))?;

        let mut results = Vec::new();
        for row in result {
            results.push(self.convert_value_fromsql(row)?);
        }

        Ok(results)
    }

    pub async fn query_opt(&self, query: &Query) -> Result<Option<Value>> {
        Ok(self.query_raw(query).await?.into_iter().next())
    }
}

trait InsertRows {
    async fn insert_rows(&self, query: &Query, rows: &Vec<Vec<Value>>) -> Result<Vec<Value>>;
}

impl InsertRows for Postgres {
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

impl SelectRows for Postgres {
    async fn select_rows(&self, query: &Query) -> Result<Vec<Value>> {
        // let (sql, params) = query.render_chunks();
        self.query_raw(query).await
    }
}

impl DataSource for Postgres {
    async fn query_fetch(&self, _query: &Query) -> Result<Vec<Map<String, Value>>> {
        let res = self.query_raw(_query).await?;
        let res = res
            .into_iter()
            .map(|v| v.as_object().unwrap().clone())
            // TODO: unwanted clone
            .collect();
        Ok(res)
    }

    async fn query_exec(&self, _query: &Query) -> Result<()> {
        todo!()
    }

    async fn query_insert(&self, _query: &Query, _rows: Vec<Vec<Value>>) -> Result<()> {
        todo!()
    }
    async fn query_row(&self, query: &Query) -> Result<Map<String, Value>> {
        let Some(Value::Object(res)) = self.query_raw(query).await?.into_iter().next() else {
            return Err(anyhow!("No rows for query_row"));
        };
        Ok(res)
    }
    async fn query_one(&self, query: &Query) -> Result<Value> {
        let Some(Value::Object(res)) = self.query_raw(query).await?.into_iter().next() else {
            return Err(anyhow!("No rows for query_one"));
        };
        let Some((_, res)) = res.into_iter().next() else {
            return Err(anyhow!("No cells in a first row of query_one"));
        };
        Ok(res)
    }
    async fn query_col(&self, query: &Query) -> Result<Vec<Value>> {
        let res = self.query_raw(query).await?;
        let res = res
            .into_iter()
            .filter_map(|v| Some(v.as_object()?.iter().next()?.1.clone()))
            .collect();
        Ok(res)
    }
}

pub struct AssociatedExpressionArc<T: DataSource> {
    pub expr: ExpressionArc,
    pub ds: T,
}

impl<T: DataSource> Deref for AssociatedExpressionArc<T> {
    type Target = ExpressionArc;

    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

impl<T: DataSource> AssociatedExpressionArc<T> {
    pub fn new(expr: ExpressionArc, ds: T) -> Self {
        Self { expr, ds }
    }
    pub async fn get_one(&self) -> Result<Value> {
        let one = self
            .ds
            .query_one(
                &Query::new().with_type(crate::sql::query::QueryType::Expression(
                    self.expr.render_chunk(),
                )),
            )
            .await?;
        Ok(one)
    }
}

/// While [`Query`] does not generally associate with the [`DataSource`], it may be inconvenient
/// to execute it. AssociatedQuery combines query with the datasource, allowing you to ealily
/// pass it around and execute it.
///
/// ```
/// let clients = Client::table();
/// let client_count = clients.count();   // returns AssociatedQuery
///
/// let cnt: Value = client_count.get_one_untuped().await?;  // actually executes the query
/// ```
///
/// AssociatedQuery can be used to make a link between DataSources:
///
/// ```
/// let clients = Client::table();
/// let client_code_query = clients.field_query(clients.code())?;
/// // returns field query (SELECT code FROM client)
///
/// let orders = Order::table();
/// let orders = orders.with_condition(
///     orders.client_code().in(orders.glue(client_code_query).await?)
/// );
/// ```
/// If Order and Client tables do share same [`DataSource`], the conditioun would be set as
///  `WHERE (client_code IN (SELECT code FROM client))`, ultimatelly saving you from
/// redundant query.
///
/// When datasources are different, [`glue()`] would execute `SELECT code FROM client`, fetch
/// the results and use those as a vector of values in a condition clause:
///  `WHERE (client_code IN [12, 13, 14])`
///
/// [`DataSource`]: crate::traits::datasource::DataSource
/// [`glue()`]: Table::glue
///
#[derive(Debug, Clone)]
pub struct AssociatedQuery<T: DataSource> {
    pub query: Query,
    pub ds: T,
}
impl<T: DataSource> Deref for AssociatedQuery<T> {
    type Target = Query;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

impl<T: DataSource> AssociatedQuery<T> {
    pub fn new(query: Query, ds: T) -> Self {
        Self { query, ds }
    }

    /// Presented with another AssociatedQuery - calculate if queries
    /// are linked with the same or different [`DataSource`]s.
    ///
    /// The same - return expression as-is.
    /// Different - execute the query and return the result as a vector of values.
    async fn glue(&self, other: AssociatedQuery<T>) -> Result<Expression> {
        if self.ds.eq(&other.ds) {
            Ok(other.query.render_chunk())
        } else {
            let vals = other.get_col_untyped().await?;
            let tpl = vec!["{}"; vals.len()].join(", ");
            Ok(Expression::new(tpl, vals))
        }
    }
}
impl<T: DataSource + Sync> Chunk for AssociatedQuery<T> {
    fn render_chunk(&self) -> Expression {
        self.query.render_chunk()
    }
}
impl<T: DataSource + Sync> ReadableDataSet<EmptyEntity> for AssociatedQuery<T> {
    async fn get_all_untyped(&self) -> Result<Vec<Map<String, Value>>> {
        self.ds.query_fetch(&self.query).await
    }

    async fn get_row_untyped(&self) -> Result<Map<String, Value>> {
        self.ds.query_row(&self.query).await
    }

    async fn get_one_untyped(&self) -> Result<Value> {
        self.ds.query_one(&self.query).await
    }

    async fn get_col_untyped(&self) -> Result<Vec<Value>> {
        self.ds.query_col(&self.query).await
    }

    async fn get(&self) -> Result<Vec<EmptyEntity>> {
        let data = self.get_all_untyped().await?;
        Ok(data
            .into_iter()
            .map(|row| serde_json::from_value(Value::Object(row)).unwrap())
            .collect())
    }

    async fn get_as<T2: serde::de::DeserializeOwned>(&self) -> Result<Vec<T2>> {
        let data = self.get_all_untyped().await?;
        Ok(data
            .into_iter()
            .map(|row| serde_json::from_value(Value::Object(row)).unwrap())
            .collect())
    }

    async fn get_some(&self) -> Result<Option<EmptyEntity>> {
        let data = self.ds.query_fetch(&self.query).await?;
        if data.len() > 0 {
            let row = data[0].clone();
            let row = serde_json::from_value(Value::Object(row)).unwrap();
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    async fn get_some_as<T2: serde::de::DeserializeOwned>(&self) -> Result<Option<T2>> {
        let data = self.ds.query_fetch(&self.query).await?;
        if data.len() > 0 {
            let row = data[0].clone();
            let row = serde_json::from_value(Value::Object(row)).unwrap();
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    fn select_query(&self) -> Query {
        self.query.clone()
    }
}

#[cfg(test)]
mod tests {

    // #[tokio::test]
    // async fn test_insert_async() {
    //     let (client, connection) = tokio_postgres::connect("host=localhost dbname=postgres", NoTls)
    //         .await
    //         .unwrap();

    //     tokio::spawn(async move {
    //         if let Err(e) = connection.await {
    //             eprintln!("connection error: {}", e);
    //         }
    //     });

    //     let postgres = Postgres::new(Arc::new(Box::new(client)));

    //     let query = Query::new()
    //         .set_table("client", None)
    //         .set_type(QueryType::Insert)
    //         .add_column_field("name")
    //         .add_column_field("email")
    //         .add_column_field("is_vip");

    //     let rows: Vec<Vec<Value>> = vec![
    //         vec![json!("John"), json!("john@gamil.com"), json!(true)],
    //         vec![json!("Jane"), json!("jave@ffs.org"), json!(true)],
    //     ];

    //     dbg!(&query.render_chunk());
    //     let ids = postgres.insert_rows(&query, &rows).await.unwrap();

    //     // should be sequential
    //     assert!(ids[0].as_i64().unwrap() + 1 == ids[1].as_i64().unwrap());
    //     let id0 = ids[0].as_i64().unwrap() as i32;
    //     let id1 = ids[1].as_i64().unwrap() as i32;

    //     let expr = expr!("id in ({}, {})", id0, id1);

    //     let delete_query = Query::new()
    //         .set_table("client", None)
    //         .set_type(QueryType::Delete)
    //         .add_condition(expr);

    //     postgres.query_raw(&delete_query).await.unwrap();
    // }
}
