use crate::query::Query;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

// Represents a dataset that may fetch all or some data
pub trait ReadableDataSet<E> {
    fn select_query(&self) -> Query;
    async fn get_all_untyped(&self) -> Result<Vec<Map<String, Value>>>;
    async fn get_row_untyped(&self) -> Result<Map<String, Value>>;
    async fn get_one_untyped(&self) -> Result<Value>;

    async fn get(&self) -> Result<Vec<E>>;
    async fn get_some(&self) -> Result<Option<E>>;
}

// // Represents a dataset that may also be modified through a query
// pub trait WritableDataSet: ReadableDataSet {
//     fn insert_query(&self) -> Query;
//     fn update_query(&self) -> Query;
//     fn delete_query(&self) -> Query;
// }
