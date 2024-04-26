use crate::query::Query;
use anyhow::Result;
use serde_json::{Map, Value};

// Represents a dataset that may generate query to fetch data, given a datasource
pub trait ReadableDataSet<'a> {
    fn select_query(&'a self) -> Query<'a>;
    async fn get_all_data(&'a self) -> Result<Vec<Map<String, Value>>>;
}

// Represents a dataset that may also be modified through a query
pub trait WritableDataSet<'a>: ReadableDataSet<'a> {
    fn insert_query(&'a self) -> Query;
    fn update_query(&'a self) -> Query;
    fn delete_query(&'a self) -> Query;
}
