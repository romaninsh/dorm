use std::future::Future;

use crate::query::Query;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

// Represents a dataset that may fetch all or some data
pub trait ReadableDataSet<E> {
    fn select_query(&self) -> Query;
    fn get_all_untyped(&self) -> impl Future<Output = Result<Vec<Map<String, Value>>>>;
    fn get_row_untyped(&self) -> impl Future<Output = Result<Map<String, Value>>>;
    fn get_one_untyped(&self) -> impl Future<Output = Result<Value>>;

    fn get(&self) -> impl Future<Output = Result<Vec<E>>>;
    fn get_some(&self) -> impl Future<Output = Result<Option<E>>>;

    fn get_as<T: DeserializeOwned>(&self) -> impl Future<Output = Result<Vec<T>>>;
    fn get_some_as<T: DeserializeOwned>(&self) -> impl Future<Output = Result<Option<T>>>;
}

// // Represents a dataset that may also be modified through a query
// pub trait WritableDataSet: ReadableDataSet {
//     fn insert_query(&self) -> Query;
//     fn update_query(&self) -> Query;
//     fn delete_query(&self) -> Query;
// }
