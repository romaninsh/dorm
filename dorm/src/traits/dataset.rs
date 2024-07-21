use crate::query::Query;
use anyhow::Result;
use serde_json::{Map, Value};

// Represents a dataset that may generate query to fetch data, given a datasource
pub trait ReadableDataSet {
    fn select_query(&self) -> Query;
    async fn get_all_data(&self) -> Result<Vec<Map<String, Value>>>;
}

// Represents a dataset that may also be modified through a query
pub trait WritableDataSet: ReadableDataSet {
    fn insert_query(&self) -> Query;
    fn update_query(&self) -> Query;
    fn delete_query(&self) -> Query;
}
