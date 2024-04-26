use anyhow::Result;
use serde_json::Map;
use serde_json::Value;

use crate::query::Query;

pub trait DataSource<'a> {
    // Provided with an arbitrary query, fetch the results and return (Value = arbytrary )
    async fn query_fetch(&self, query: &Query<'a>) -> Result<Vec<Map<String, Value>>>;

    // Execute a query without returning any results (e.g. DELETE, UPDATE, ALTER, etc.)
    async fn query_exec(&self, query: &Query<'a>) -> Result<()>;

    // Insert ordered list of rows into a table as described by query columns
    async fn query_insert(&self, query: &Query<'a>, rows: Vec<Vec<Value>>) -> Result<()>;
}
