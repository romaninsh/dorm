use crate::condition::Condition;
use crate::query::Query;
use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;
use crate::traits::sql_chunk::SqlChunk;
use crate::{Expression, Field};
use anyhow::Result;
use indexmap::IndexMap;
use serde_json::{Map, Value};

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.
pub struct Table<'a, T: DataSource<'a>> {
    data_source: &'a T,
    table_name: String,
    fields: IndexMap<String, Field>,
    title_field: Option<String>,
    conditions: Vec<Condition<'a>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, T: DataSource<'a>> Table<'a, T> {
    pub fn new(table_name: &str, data_source: &'a T) -> Table<'a, T> {
        Table {
            table_name: table_name.to_string(),
            data_source,
            title_field: None,
            conditions: Vec::new(),
            fields: IndexMap::new(),
            _marker: std::marker::PhantomData,
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

    pub fn add_condition(
        mut self,
        field: &'static str,
        op: &'static str,
        value: Box<dyn SqlChunk<'a>>,
    ) -> Self {
        self.conditions.push(Condition::new(field, op, value));
        self
    }

    pub fn get_select_query<'b>(&'a self) -> Query<'b>
    where
        'b: 'a, // 'a is longer than 'b
    {
        let mut query = Query::new(&self.table_name);
        for (field, _) in &self.fields {
            let field_object = Field::new(field.clone());
            query = query.add_column(field.clone(), Box::new(field_object));
        }
        for condition in self.conditions.iter() {
            query = query.add_condition(Box::new(condition));
        }
        query
    }

    pub async fn get_all_data(&'a self) -> Result<Vec<Map<String, Value>>> {
        self.data_source.query_fetch(&self.get_select_query()).await
    }

    pub fn sum(&'a self, expr: &str) -> Expression<'a> {
        let field = self.fields.get(expr).unwrap();
        Expression::new("SUM({})".to_string(), vec![field])
    }
}
impl<'a, T: DataSource<'a>> ReadableDataSet<'a> for Table<'a, T> {
    fn select_query(&'a self) -> Query<'a> {
        self.get_select_query()
    }

    async fn get_all_data(&'a self) -> Result<Vec<Map<String, Value>>> {
        let q = self.select_query();
        let x = self.data_source.query_fetch(&q).await;
        x
    }
}

impl<'a, T: DataSource<'a>> WritableDataSet<'a> for Table<'a, T> {
    fn insert_query(&'a self) -> Query {
        todo!()
    }

    fn update_query(&'a self) -> Query {
        todo!()
    }

    fn delete_query(&'a self) -> Query {
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

        let table = Table::new("users", &data_source)
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

        let table = Table::new("users", &data_source)
            .add_field("name")
            .add_field("surname")
            .add_condition("name", "=", Box::new("John".to_owned()));

        let query = table.get_select_query().render_chunk().split();

        assert_eq!(query.0, "SELECT name, surname FROM users WHERE name = {}");
        assert_eq!(query.1[0], json!("John"));

        let result = table.get_all_data().await;

        assert_eq!(result.unwrap(), *data_source.data());
    }

    #[test]
    fn test_vip_client() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let vip_client = Table::new("client", &db)
            .add_title_field("name")
            .add_field("is_vip")
            .add_field("total_spent")
            .add_condition("is_vip", "is", Box::new("true".to_owned()));

        let sum = vip_client.sum("total_spent");
        assert_eq!(
            sum.render_chunk().sql().deref(),
            "(SUM(total_spent))".to_owned()
        );
    }
}
