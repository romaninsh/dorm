use anyhow::Result;
use serde_json::Map;
use serde_json::Value;

use crate::query::Query;

pub trait DataSource {
    // Provided with an arbitrary query, fetch the results and return (Value = arbytrary )
    fn query_fetch(&self, query: &Query) -> Result<Vec<Map<String, Value>>>;

    // Execute a query without returning any results (e.g. DELETE, UPDATE, ALTER, etc.)
    fn query_exec(&self, query: &Query) -> Result<()>;

    // Insert ordered list of rows into a table as described by query columns
    fn query_insert(&self, query: &Query, rows: Vec<Vec<Value>>) -> Result<()>;
}
