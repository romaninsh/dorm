#![allow(async_fn_in_trait)]

use anyhow::Result;
use serde_json::Map;
use serde_json::Value;

use crate::query::Query;

pub trait DataSource: Clone {
    // Provided with an arbitrary query, fetch the results and return (Value = arbytrary )
    async fn query_fetch(&self, query: &Query) -> Result<Vec<Map<String, Value>>>;

    // Execute a query without returning any results (e.g. DELETE, UPDATE, ALTER, etc.)
    async fn query_exec(&self, query: &Query) -> Result<()>;

    // Insert ordered list of rows into a table as described by query columns
    async fn query_insert(&self, query: &Query, rows: Vec<Vec<Value>>) -> Result<()>;

    async fn query_one(&self, query: &Query) -> Result<Value>;
}
