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

    async fn update_with<F, T2>(&self, values: T2) -> Result<()>
    where
        T2: Serialize + Clone,
    {
        // ensure that T2 does not specify ID field
        let Value::Object(values_map) = serde_json::to_value(values.clone()).unwrap() else {
            return Err(anyhow::anyhow!("T2 must be a struct"));
        };

        if let Some(ref id_field) = self.id_field {
            if values_map.get(id_field).is_some() {
                return Err(anyhow::anyhow!("T2 must not specify ID field"));
            }
        }

        let query = self.get_update_query(values);
        self.data_source.query_exec(&query).await
    }

    async fn delete(&self) -> Result<()> {
        let query = self.get_empty_query().with_type(QueryType::Delete);
        self.data_source.query_exec(&query).await
    }
}
