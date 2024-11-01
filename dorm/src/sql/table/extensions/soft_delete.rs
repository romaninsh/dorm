use std::sync::Arc;

use serde_json::json;

use crate::{
    prelude::{AnyTable, Entity, Table},
    sql::{Field, Operations, Query},
    traits::datasource::DataSource,
};

use super::TableExtension;

pub struct SoftDelete {
    soft_delete_field: String,
}

impl SoftDelete {
    fn new(soft_delete_field: &str) -> Self {
        SoftDelete {
            soft_delete_field: soft_delete_field.to_string(),
        }
    }
    fn is_deleted(&self, table: Arc<Box<dyn AnyTable>>) -> Arc<Field> {
        table.get_field(&self.soft_delete_field).unwrap()
    }
}

impl TableExtension for SoftDelete {
    /// When selecting records, exclude deleted records
    fn before_select_query(&self, table: Arc<Box<dyn AnyTable>>, query: Query) -> Query {
        query.with_condition(self.is_deleted(table).eq(&false))
    }
    /// When deleting records, mark them as deleted instead
    fn before_delete_query(&self, _table: Arc<Box<dyn AnyTable>>, query: Query) -> Query {
        query
            .with_type(crate::sql::query::QueryType::Update)
            .with_set_field(&self.soft_delete_field, json!(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mocks::datasource::MockDataSource,
        prelude::{Chunk, Operations},
        sql::table::extensions::Hooks,
    };

    #[tokio::test]
    async fn test_soft_delete() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let data_source = MockDataSource::new(&data);

        let mut table = Table::new("users", data_source.clone())
            .with_field("name")
            .with_field("surname");

        let table: Arc<Box<dyn AnyTable>> = Arc::new(Box::new(table));

        let mut ext = Hooks::new();
        ext.add_hook(table, Box::new(SoftDelete::new("is_deleted")));

        table.add_condition(table.get_field("name").unwrap().eq(&"John".to_string()));

        let query = table.get_select_query();
        let query = ext.before_select_query(Arc::new(Box::new(table)), query);

        let result = query.render_chunk().split();

        assert_eq!(
            result.0,
            "SELECT name, surname FROM users WHERE (name = {}) AND (is_deleted = {})"
        );
        assert_eq!(result.1[0], json!("John"));
        assert_eq!(result.1[1], json!(false));
    }
}
