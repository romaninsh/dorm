use crate::{
    dataset::WritableDataSet, prelude::Entity, query::QueryType, traits::datasource::DataSource,
};

use super::Table;
use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Map, Value};

// You should be able to insert and delete data in a table
impl<T: DataSource, E: Entity> WritableDataSet<E> for Table<T, E> {
    async fn insert(&self, record: E) -> Result<()> {
        let query = self.get_insert_query(record);
        self.data_source.query_exec(&query).await
    }

    async fn update<F>(&self, f: F) -> Result<()> {
        todo!()
    }

    async fn delete(&self) -> Result<()> {
        let query = self.get_empty_query().with_type(QueryType::Delete);
        self.data_source.query_exec(&query).await
    }
}
