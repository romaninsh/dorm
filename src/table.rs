use std::sync::Arc;

use crate::condition::Condition;
use crate::expr_arc;
use crate::expression::ExpressionArc;
use crate::field::Field;
// use crate::join::Join;
use crate::prelude::{AssociatedQuery, Operations};
use crate::query::{Query, QueryType};
use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;
use crate::traits::sql_chunk::SqlChunk;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::{Map, Value};

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.

#[derive(Clone)]
pub struct Table<T: DataSource> {
    data_source: T,
    table_name: String,
    table_alias: Option<String>,
    // joins: Vec<Join<T>>,
    fields: IndexMap<String, Field>,
    title_field: Option<String>,
    conditions: Vec<Condition>,
}

impl<T: DataSource> Table<T> {
    pub fn new(table_name: &str, data_source: T) -> Table<T> {
        Table {
            table_name: table_name.to_string(),
            table_alias: None,
            // joins: Vec::new(),
            data_source,
            title_field: None,
            conditions: Vec::new(),
            fields: IndexMap::new(),
        }
    }

    fn new_field(&self, field: String) -> Field {
        Field::new(field, self.table_alias.clone())
    }

    pub fn get_field(&self, field: &str) -> &Field {
        self.fields.get(field).unwrap()
    }

    pub fn fields(&self) -> &IndexMap<String, Field> {
        &self.fields
    }

    pub fn add_field(mut self, field: &str) -> Self {
        self.fields
            .insert(field.to_string(), self.new_field(field.to_string()));
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
        self,
        field: &'static str,
        op: &'static str,
        value: impl SqlChunk + 'static,
    ) -> Result<Self> {
        let field = self
            .fields
            .get(field)
            .ok_or_else(|| anyhow!("Field not found: {}", field))?
            .clone();
        Ok(self.add_condition(Condition::from_field(field, op, Arc::new(Box::new(value)))))
    }

    pub fn has_many_cb(self, relation: &str, cb: impl FnOnce() -> Table<T>) -> Self {
        todo!()
    }
    pub fn has_one_cb(self, relation: &str, cb: impl FnOnce() -> Table<T>) -> Self {
        todo!()
    }

    pub fn get_ref(&self, field: &str) -> Table<T> {
        todo!()
    }

    pub fn add_field_cb(
        self,
        field: &str,
        cb: impl FnOnce(&Table<T>) -> Box<dyn SqlChunk>,
    ) -> Self {
        todo!();
        // let field = cb();
        // self.fields.insert(field.name().clone(), field);
        // self
    }

    pub fn id(&self) -> Field {
        // Field::new("test".to_string(), Some("test".to_string()))
        self.fields.get("id").unwrap().clone()
    }
    pub fn with_id(self, id: Value) -> Self {
        let f = self.id().eq(&id);
        self.add_condition(f)
    }

    /*
    // TODO: Need to allow self-join and provide unique alias
    pub fn add_join(mut self, join: Join<T>) -> Self {
        self.joins.push(join);
        self
    }

    pub fn add_join_table(
        mut self,
        our_field: String,
        other_table: String,
        other_field: String,
    ) -> Join<T> {
        let join = Join::new(other_table);
        let joined_field = join.add_field(other_field);
        let condition = joined_field.eq(self.fields.get(our_field));
        join.add_on_condition(condition);
        join
    }
    */

    pub fn get_empty_query(&self) -> Query {
        let mut query = Query::new().set_table(&self.table_name);
        for condition in self.conditions.iter() {
            query = query.add_condition(condition.clone());
        }
        query
    }

    pub fn get_select_query(&self) -> Query {
        let mut query = Query::new().set_table(&self.table_name);
        for (field_key, field_val) in &self.fields {
            query = query.add_column(field_key.clone(), field_val.clone());
        }
        for condition in self.conditions.iter() {
            query = query.add_condition(condition.clone());
        }
        query
    }

    pub fn get_insert_query(&self) -> Query {
        let mut query = Query::new()
            .set_table(&self.table_name)
            .set_type(QueryType::Insert);
        for (field, _) in &self.fields {
            let field_object = Field::new(field.clone(), self.table_alias.clone());
            query = query.add_column(field.clone(), field_object);
        }
        query
    }

    pub async fn get_all_data(&self) -> Result<Vec<Map<String, Value>>> {
        self.data_source.query_fetch(&self.get_select_query()).await
    }

    pub fn sum(&self, field: &Field) -> AssociatedQuery<T> {
        let query = self
            .get_empty_query()
            .add_column("sum".to_string(), expr_arc!("SUM({})", field.clone()));
        AssociatedQuery::new(query, self.data_source.clone())
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

pub trait TableDelegate<T: DataSource> {
    fn table(&self) -> &Table<T>;

    fn id(&self) -> Field {
        self.table().id().clone()
    }
    fn add_condition(&self, condition: Condition) -> Table<T> {
        self.table().clone().add_condition(condition)
    }
    fn sum(&self, field: &Field) -> AssociatedQuery<T> {
        self.table().sum(field)
    }
}

#[cfg(test)]
mod tests {

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
            .add_condition_on_field("name", "=", "John".to_owned())
            .unwrap();

        let query = table.get_select_query().render_chunk().split();

        assert_eq!(query.0, "SELECT name, surname FROM users WHERE (name = {})");
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
            .add_condition_on_field("is_vip", "is", "true".to_owned())
            .unwrap();

        let sum = vip_client.sum(vip_client.fields.get("total_spent").unwrap());
        assert_eq!(
            sum.render_chunk().sql().clone(),
            "SELECT (SUM(total_spent)) AS sum FROM client WHERE (is_vip is {})".to_owned()
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
            "INSERT INTO users (name, surname) VALUES ({}, {}) returning id"
        );
        assert_eq!(query.1[0], Value::Null);
        assert_eq!(query.1[1], Value::Null);
    }
}
