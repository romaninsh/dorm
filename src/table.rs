use std::sync::Arc;

use crate::condition::Condition;
use crate::expr_arc;
use crate::expression::ExpressionArc;
use crate::field::Field;
use crate::query::{Query, QueryType};
use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;
use crate::traits::sql_chunk::SqlChunk;
use anyhow::Result;
use indexmap::IndexMap;
use serde_json::{Map, Value};

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.
pub struct Table<T: DataSource> {
    data_source: T,
    table_name: String,
    fields: IndexMap<String, Field>,
    title_field: Option<String>,
    conditions: Vec<Condition>,
}

impl<T: DataSource> Table<T> {
    pub fn new(table_name: &str, data_source: T) -> Table<T> {
        Table {
            table_name: table_name.to_string(),
            data_source,
            title_field: None,
            conditions: Vec::new(),
            fields: IndexMap::new(),
        }
    }

    pub fn id(&self) -> &Field {
        self.fields.get("id").unwrap()
    }

    pub fn fields(&self) -> &IndexMap<String, Field> {
        &self.fields
    }

    pub fn add_field(mut self, field: &str) -> Self {
        self.fields
            .insert(field.to_string(), Field::new(field.to_string()));
        self
    }

    pub fn add_title_field(mut self, field: &str) -> Self {
        self.title_field = Some(field.to_string());
        self.add_field(field)
    }

    pub fn add_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn add_condition_on_field(
        mut self,
        field: &'static str,
        op: &'static str,
        value: impl SqlChunk + 'static,
    ) -> Self {
        self.add_condition(Condition::new(field, op, Arc::new(Box::new(value))))
    }

    pub fn get_select_query(&self) -> Query {
        let mut query = Query::new(&self.table_name);
        for (field, _) in &self.fields {
            let field_object = Field::new(field.clone());
            query = query.add_column(field.clone(), field_object);
        }
        for condition in self.conditions.iter() {
            query = query.add_condition(condition.clone());
        }
        query
    }

    pub fn get_insert_query(&self) -> Query {
        let mut query = Query::new(&self.table_name).set_type(QueryType::Insert);
        for (field, _) in &self.fields {
            let field_object = Field::new(field.clone());
            query = query.add_column(field.clone(), field_object);
        }
        query
    }

    pub async fn get_all_data(&self) -> Result<Vec<Map<String, Value>>> {
        self.data_source.query_fetch(&self.get_select_query()).await
    }

    pub fn sum(&self, expr: &str) -> ExpressionArc {
        let field = self.fields.get(expr).unwrap();
        expr_arc!("SUM({})", field.clone())
    }
}

impl<T: DataSource> ReadableDataSet for Table<T> {
    fn select_query(&self) -> Query {
        self.get_select_query()
    }

    async fn get_all_data(&self) -> Result<Vec<Map<String, Value>>> {
        let q = self.select_query();
        let x = self.data_source.query_fetch(&q).await;
        x
    }
}

impl<T: DataSource> WritableDataSet for Table<T> {
    fn insert_query(&self) -> Query {
        todo!()
    }

    fn update_query(&self) -> Query {
        todo!()
    }

    fn delete_query(&self) -> Query {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use serde_json::json;

    use super::*;
    use crate::mocks::datasource::MockDataSource;

    #[tokio::test]
    async fn test_table() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);

        let data_source = MockDataSource::new(&data);

        let table = Table::new("users", data_source.clone())
            .add_field("name")
            .add_field("surname");

        let result = table.get_all_data().await;

        assert_eq!(result.unwrap().clone(), *data_source.data());
    }

    #[tokio::test]
    async fn test_conditions() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let data_source = MockDataSource::new(&data);

        let table = Table::new("users", data_source.clone())
            .add_field("name")
            .add_field("surname")
            .add_condition_on_field("name", "=", "John".to_owned());

        let query = table.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT `name`, `surname` FROM `users` WHERE name = {}"
        );
        assert_eq!(query.1[0], json!("John"));

        let result = table.get_all_data().await;

        assert_eq!(result.unwrap(), *data_source.data());
    }

    #[test]
    fn test_vip_client() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let vip_client = Table::new("client", db)
            .add_title_field("name")
            .add_field("is_vip")
            .add_field("total_spent")
            .add_condition_on_field("is_vip", "is", "true".to_owned());

        let sum = vip_client.sum("total_spent");
        assert_eq!(
            sum.render_chunk().sql().deref(),
            "SUM(`total_spent`)".to_owned()
        );
    }

    #[test]
    fn test_insert_query() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let table = Table::new("users", db)
            .add_field("name")
            .add_field("surname");

        let query = table.get_insert_query().render_chunk().split();

        assert_eq!(
            query.0,
            "INSERT INTO users (`name`, `surname`) VALUES ({}, {})"
        );
        assert_eq!(query.1[0], Value::Null);
        assert_eq!(query.1[1], Value::Null);
    }
}
