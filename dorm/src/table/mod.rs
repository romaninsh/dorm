use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::condition::Condition;
use crate::expr_arc;
use crate::expression::ExpressionArc;
use crate::field::Field;
use crate::join::Join;
use crate::lazy_expression::LazyExpression;
use crate::prelude::{AssociatedQuery, Expression};
use crate::query::Query;
use crate::reference::RelatedReference;
use crate::traits::any::{AnyTable, RelatedTable};
use crate::traits::datasource::DataSource;
use crate::traits::entity::{EmptyEntity, Entity};
use crate::uniqid::UniqueIdVendor;
use anyhow::Result;
use indexmap::IndexMap;
use serde_json::{Map, Value};

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.

#[derive(Debug)]
pub struct Table<T: DataSource, E: Entity> {
    data_source: T,
    _phantom: std::marker::PhantomData<E>,

    table_name: String,
    table_alias: Option<String>,
    id_field: Option<String>,
    title_field: Option<String>,

    conditions: Vec<Condition>,
    fields: IndexMap<String, Arc<Field>>,
    joins: IndexMap<String, Arc<Join<T>>>,
    lazy_expressions: IndexMap<String, LazyExpression<T, E>>,
    refs: IndexMap<String, RelatedReference<T, E>>,
    table_aliases: Arc<Mutex<UniqueIdVendor>>,
}

mod with_fetching;
mod with_fields;
mod with_joins;
mod with_queries;
mod with_refs;

impl<T: DataSource + Clone, E: Entity> Clone for Table<T, E> {
    fn clone(&self) -> Self {
        Table {
            data_source: self.data_source.clone(),
            _phantom: self._phantom.clone(),

            table_name: self.table_name.clone(),
            table_alias: self.table_alias.clone(),
            id_field: self.id_field.clone(),
            title_field: self.title_field.clone(),

            conditions: self.conditions.clone(),
            fields: self.fields.clone(),
            joins: self.joins.clone(),
            lazy_expressions: self.lazy_expressions.clone(),
            refs: self.refs.clone(),

            // Perform a deep clone of the UniqueIdVendor
            table_aliases: Arc::new(Mutex::new((*self.table_aliases.lock().unwrap()).clone())),
        }
    }
}

impl<T: DataSource, E: Entity> AnyTable for Table<T, E> {
    fn as_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn get_field(&self, name: &str) -> Option<Arc<Field>> {
        self.fields.get(name).cloned()
    }
    fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }
}

impl<T: DataSource, E: Entity> RelatedTable<T> for Table<T, E> {
    fn field_query(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        let query = self.get_empty_query().with_column(field.name(), field);
        AssociatedQuery::new(query, self.data_source.clone())
    }

    // TODO: debug why this overwrites the previous fields
    fn add_fields_into_query(&self, mut query: Query, alias_prefix: Option<&str>) -> Query {
        for (field_key, field_val) in &self.fields {
            let field_val = if let Some(alias_prefix) = &alias_prefix {
                let alias = format!("{}_{}", alias_prefix, field_key);
                let mut field_val = field_val.deref().clone();
                field_val.set_field_alias(alias);
                Arc::new(field_val)
            } else {
                field_val.clone()
            };
            query = query.with_column(
                field_val
                    .deref()
                    .get_field_alias()
                    .unwrap_or_else(|| field_key.clone()),
                field_val,
            );
        }

        for (alias, join) in &self.joins {
            query = join.add_fields_into_query(query, Some(alias));
        }

        query
    }

    fn get_alias(&self) -> Option<&String> {
        self.table_alias.as_ref()
    }
    fn set_alias(&mut self, alias: &str) {
        if let Some(alias) = &self.table_alias {
            self.table_aliases.lock().unwrap().dont_avoid(alias);
        }
        self.table_alias = Some(alias.to_string());
        self.table_aliases.lock().unwrap().avoid(alias);
        for field in self.fields.values_mut() {
            let mut new_field = field.deref().deref().clone();
            new_field.set_table_alias(alias.to_string());
            *field = Arc::new(new_field);
        }
        for condition in &mut self.conditions {
            condition.set_table_alias(alias);
        }
    }
    fn get_table_name(&self) -> Option<&String> {
        Some(&self.table_name)
    }
    fn get_fields(&self) -> &IndexMap<String, Arc<Field>> {
        &self.fields
    }
    fn get_join(&self, table_alias: &str) -> Option<Arc<Join<T>>> {
        self.joins.get(table_alias).map(|r| r.clone())
    }
}

impl<T: DataSource, E: Entity> Table<T, E> {
    pub fn new_with_entity(table_name: &str, data_source: T) -> Table<T, E> {
        Table {
            data_source,
            _phantom: std::marker::PhantomData,

            table_name: table_name.to_string(),
            table_alias: None,
            id_field: None,
            title_field: None,

            conditions: Vec::new(),
            fields: IndexMap::new(),
            joins: IndexMap::new(),
            lazy_expressions: IndexMap::new(),
            refs: IndexMap::new(),
            table_aliases: Arc::new(Mutex::new(UniqueIdVendor::new())),
        }
    }
}

impl<T: DataSource> Table<T, EmptyEntity> {
    pub fn new(table_name: &str, data_source: T) -> Table<T, EmptyEntity> {
        Table {
            data_source,
            _phantom: std::marker::PhantomData,

            table_name: table_name.to_string(),
            table_alias: None,
            id_field: None,
            title_field: None,

            conditions: Vec::new(),
            fields: IndexMap::new(),
            joins: IndexMap::new(),
            lazy_expressions: IndexMap::new(),
            refs: IndexMap::new(),
            table_aliases: Arc::new(Mutex::new(UniqueIdVendor::new())),
        }
    }
}

impl<T: DataSource, E: Entity> Table<T, E> {
    /// Use a callback with a builder pattern:
    /// ```
    /// let books = BookSet::new().with(|b| {
    ///    b.add_field("title");
    ///    b.add_field("price");
    /// }).with(|b| {
    ///    b.add_condition(b.get_field("title").unwrap().gt(100));
    /// });
    /// ```
    pub fn with<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        func(&mut self);
        self
    }

    pub fn into_entity<E2: Entity>(self) -> Table<T, E2> {
        Table {
            data_source: self.data_source,
            _phantom: std::marker::PhantomData,

            table_name: self.table_name,
            table_alias: self.table_alias,
            id_field: self.id_field,
            title_field: self.title_field,

            conditions: self.conditions,
            fields: self.fields,
            joins: IndexMap::new(),            // TODO: cast proprely
            lazy_expressions: IndexMap::new(), // TODO: cast proprely
            refs: IndexMap::new(),             // TODO: cast proprely

            // Perform a deep clone of the UniqueIdVendor
            table_aliases: Arc::new(Mutex::new((*self.table_aliases.lock().unwrap()).clone())),
        }
    }

    pub fn with_alias(mut self, alias: &str) -> Self {
        self.set_alias(alias);
        self
    }

    /// Add a condition to the table, limiting what records
    /// the DataSet will represent
    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    /// A handy way to add a condition during table building:
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.add_condition(condition);
        self
    }

    // ---- Expressions ----
    //  BeforeQuery(Arc<Box<dyn Fn(&Query) -> Expression>>),
    pub fn add_expression(
        &mut self,
        name: &str,
        expression: impl Fn(&Table<T, E>) -> Expression + 'static + Sync + Send,
    ) {
        self.lazy_expressions.insert(
            name.to_string(),
            LazyExpression::BeforeQuery(Arc::new(Box::new(expression))),
        );
    }

    pub fn with_expression(
        mut self,
        name: &str,
        expression: impl Fn(&Table<T, E>) -> Expression + 'static + Sync + Send,
    ) -> Self {
        self.add_expression(name, expression);
        self
    }

    pub async fn get_all_data(&self) -> Result<Vec<Map<String, Value>>> {
        self.data_source.query_fetch(&self.get_select_query()).await
    }

    pub fn sum(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        let query = self
            .get_empty_query()
            .with_column("sum".to_string(), expr_arc!("SUM({})", field));
        AssociatedQuery::new(query, self.data_source.clone())
    }

    pub fn count(&self) -> AssociatedQuery<T> {
        let query = self
            .get_empty_query()
            .with_column("count".to_string(), expr_arc!("COUNT(*)"));
        AssociatedQuery::new(query, self.data_source.clone())
    }
}

// impl<T: DataSource, E: Entity> WritableDataSet for Table<T, E> {
//     fn insert_query(&self) -> Query {
//         todo!()
//     }

//     fn update_query(&self) -> Query {
//         todo!()
//     }

//     fn delete_query(&self) -> Query {
//         todo!()
//     }
// }

pub trait TableDelegate<T: DataSource, E: Entity> {
    fn table(&self) -> &Table<T, E>;

    fn id(&self) -> Arc<Field> {
        self.table().id()
    }
    fn add_condition(&self, condition: Condition) -> Table<T, E> {
        self.table().clone().with_condition(condition)
    }
    fn sum(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        self.table().sum(field)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use serde_json::json;

    use super::*;
    use crate::{
        mocks::datasource::MockDataSource,
        prelude::{Operations, SqlChunk},
    };

    #[tokio::test]
    async fn test_table() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);

        let data_source = MockDataSource::new(&data);

        let table = Table::new("users", data_source.clone())
            .with_field("name")
            .with_field("surname");

        let result = table.get_all_data().await;

        assert_eq!(result.unwrap().clone(), *data_source.data());
    }

    #[tokio::test]
    async fn test_with() {
        let data = json!([]);
        let data_source = MockDataSource::new(&data);
        let books = Table::new("book", data_source)
            .with(|b| {
                b.add_field("title".to_string(), Field::new("title".to_string(), None));
                b.add_field("price".to_string(), Field::new("price".to_string(), None));
            })
            .with(|b| {
                b.add_condition(b.get_field("title").unwrap().gt(100));
            });

        let query = books.get_select_query().render_chunk().split();

        assert_eq!(query.0, "SELECT title, price FROM book WHERE (title > {})");
    }

    #[tokio::test]
    async fn test_conditions() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let data_source = MockDataSource::new(&data);

        let table = Table::new("users", data_source.clone())
            .with_field("name")
            .with_field("surname")
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
            .with_title_field("name")
            .with_field("is_vip")
            .with_field("total_spent")
            .add_condition_on_field("is_vip", "is", "true".to_owned())
            .unwrap();

        let sum = vip_client.sum(vip_client.get_field("total_spent").unwrap());
        assert_eq!(
            sum.render_chunk().sql().clone(),
            "SELECT (SUM(total_spent)) AS sum FROM client WHERE (is_vip is {})".to_owned()
        );
    }
}
