use crate::dataset::ReadableDataSet;
use crate::sql::Query;
use crate::table::Table;
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

/// Implementing fetching methods for table, including
/// combinations of query building and executing for
/// single or multiple rows.

impl<T: DataSource, E: Entity> ReadableDataSet<E> for Table<T, E> {
    fn select_query(&self) -> Query {
        self.get_select_query()
    }

    async fn get_all_untyped(&self) -> Result<Vec<Map<String, Value>>> {
        let query = self.select_query();
        self.data_source.query_fetch(&query).await
    }

    async fn get_row_untyped(&self) -> Result<Map<String, Value>> {
        let query = self.select_query();
        self.data_source.query_row(&query).await
    }

    async fn get_col_untyped(&self) -> Result<Vec<Value>> {
        let query = self.select_query();
        self.data_source.query_col(&query).await
    }

    async fn get_one_untyped(&self) -> Result<Value> {
        let query = self.select_query();
        self.data_source.query_one(&query).await
    }

    async fn get(&self) -> Result<Vec<E>> {
        let data = self.get_all_untyped().await?;
        Ok(data
            .into_iter()
            .map(|row| serde_json::from_value(Value::Object(row)).unwrap())
            .collect())
    }

    async fn get_as<T2: DeserializeOwned>(&self) -> Result<Vec<T2>> {
        let data = self.get_all_untyped().await?;
        Ok(data
            .into_iter()
            .map(|row| serde_json::from_value(Value::Object(row)).unwrap())
            .collect())
    }

    async fn get_some(&self) -> Result<Option<E>> {
        let query = self.select_query();
        let data = self.data_source.query_fetch(&query).await?;
        if data.len() > 0 {
            let row = data[0].clone();
            let row = serde_json::from_value(Value::Object(row)).unwrap();
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    async fn get_some_as<T2: DeserializeOwned>(&self) -> Result<Option<T2>> {
        let query = self.select_query();
        let data = self.data_source.query_fetch(&query).await?;
        if data.len() > 0 {
            let row = data[0].clone();
            let row = serde_json::from_value(Value::Object(row)).unwrap();
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }
}

// impl<T: DataSource, E: Entity> Table<T, E> {
// }

#[cfg(test)]
mod tests {
    // use super::*;
}
