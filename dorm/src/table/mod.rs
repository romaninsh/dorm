use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::condition::Condition;
use crate::expr_arc;
use crate::expression::ExpressionArc;
use crate::field::Field;
use crate::join::Join;
use crate::lazy_expression::LazyExpression;
use crate::prelude::{AssociatedQuery, Expression};
use crate::query::{Query, QueryType};
use crate::reference::Reference;
use crate::traits::column::Column;
use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;
use crate::uniqid::UniqueIdVendor;
use anyhow::Result;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{to_value, Map, Value};

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.

#[derive(Debug)]
pub struct Table<T: DataSource> {
    data_source: T,

    table_name: String,
    table_alias: Option<String>,
    id_field: Option<String>,
    title_field: Option<String>,

    conditions: Vec<Condition>,
    fields: IndexMap<String, Arc<Field>>,
    joins: IndexMap<String, Arc<Join<T>>>,
    lazy_expressions: IndexMap<String, LazyExpression<T>>,
    refs: IndexMap<String, Reference<T>>,

    table_aliases: Arc<Mutex<UniqueIdVendor>>,
}

mod with_fields;
mod with_joins;
mod with_refs;

impl<T: DataSource + Clone> Clone for Table<T> {
    fn clone(&self) -> Self {
        Table {
            data_source: self.data_source.clone(),

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

impl<T: DataSource> Table<T> {
    pub fn new(table_name: &str, data_source: T) -> Table<T> {
        Table {
            data_source,

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

    pub fn set_alias(&mut self, alias: &str) {
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

    pub fn with_alias(mut self, alias: &str) -> Self {
        self.set_alias(alias);
        self
    }

    pub fn get_alias(&self) -> Option<&String> {
        self.table_alias.as_ref()
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
    pub fn add_expression_before_query(
        &mut self,
        name: &str,
        expression: impl Fn(&Table<T>) -> Expression + 'static + Sync + Send,
    ) {
        self.lazy_expressions.insert(
            name.to_string(),
            LazyExpression::BeforeQuery(Arc::new(Box::new(expression))),
        );
    }

    pub fn get_empty_query(&self) -> Query {
        let mut query = Query::new().set_table(&self.table_name, self.table_alias.clone());
        for condition in self.conditions.iter() {
            query = query.add_condition(condition.clone());
        }
        for (alias, join) in &self.joins {
            query = query.add_join(join.join_query().clone());
        }
        query
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
            query = query.add_column(
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

    pub fn get_select_query(&self) -> Query {
        let mut query = self.get_empty_query();
        query = self.add_fields_into_query(query, None);
        query
    }

    pub fn get_select_query_for_fields(
        &self,
        fields: IndexMap<String, Arc<Box<dyn Column>>>,
    ) -> Query {
        let mut query = Query::new().set_table(&self.table_name, self.table_alias.clone());
        for (field_alias, field_val) in fields {
            let field_val = field_val.clone();
            query = query.add_column_arc(field_alias, field_val);
        }
        query
    }

    pub fn get_select_query_for_field_names(&self, field_names: &[&str]) -> Query {
        let mut index_map = IndexMap::new();
        for field_name in field_names {
            let field = self.search_for_field(field_name).unwrap();
            index_map.insert(field_name.to_string(), Arc::new(field));
        }
        self.get_select_query_for_fields(index_map)
    }

    pub fn get_select_query_for_struct<R: Serialize>(&self, default: R) -> Query {
        let json_value = to_value(default).unwrap();

        let field_names = match json_value {
            Value::Object(map) => {
                let field_names = map.keys().cloned().collect::<Vec<String>>();
                field_names
            }
            _ => panic!("Expected argument to be a struct"),
        };

        let i = field_names
            .into_iter()
            .filter_map(|f| self.search_for_field(&f).map(|c| (f, Arc::new(c))));

        let i = i.collect::<IndexMap<_, _>>();

        self.get_select_query_for_fields(i)
    }

    pub fn get_insert_query(&self) -> Query {
        let mut query = Query::new()
            .set_table(&self.table_name, None)
            .set_type(QueryType::Insert);
        for (field, _) in &self.fields {
            let field_object = Arc::new(Field::new(field.clone(), self.table_alias.clone()));
            query = query.add_column(field.clone(), field_object);
        }
        query
    }

    pub async fn get_all_data(&self) -> Result<Vec<Map<String, Value>>> {
        self.data_source.query_fetch(&self.get_select_query()).await
    }

    pub fn field_query(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        let query = self.get_empty_query().add_column(field.name(), field);
        AssociatedQuery::new(query, self.data_source.clone())
    }

    pub fn sum(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        let query = self
            .get_empty_query()
            .add_column("sum".to_string(), expr_arc!("SUM({})", field));
        AssociatedQuery::new(query, self.data_source.clone())
    }

    pub fn count(&self) -> AssociatedQuery<T> {
        let query = self
            .get_empty_query()
            .add_column("count".to_string(), expr_arc!("COUNT(*)"));
        AssociatedQuery::new(query, self.data_source.clone())
    }

    pub fn search_for_field(&self, field_name: &str) -> Option<Box<dyn Column>> {
        // perhaps we have a field like this
        // let field = self.get_field(field_name);

        if let Some(field) = self.get_field(field_name) {
            return Some(Box::new(field));
        }

        // maybe joined table have a field we want
        for (_, join) in self.joins.iter() {
            if let Some(field) = join.table().get_field(field_name) {
                return Some(Box::new(field));
            }
        }

        // maybe we have a lazy expression
        if let Some(lazy_expression) = self.lazy_expressions.get(field_name) {
            return match lazy_expression {
                LazyExpression::AfterQuery(_) => None,
                LazyExpression::BeforeQuery(expr) => {
                    let x = (expr)(self);
                    Some(Box::new(x))
                }
            };
        }
        None
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

    fn id(&self) -> Arc<Field> {
        self.table().id()
    }
    fn add_condition(&self, condition: Condition) -> Table<T> {
        self.table().clone().with_condition(condition)
    }
    fn sum(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        self.table().sum(field)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use serde::{Deserialize, Serialize};
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

    #[test]
    fn test_insert_query() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let table = Table::new("users", db)
            .with_field("name")
            .with_field("surname");

        let query = table.get_insert_query().render_chunk().split();

        assert_eq!(
            query.0,
            "INSERT INTO users (name, surname) VALUES ({}, {}) returning id"
        );
        assert_eq!(query.1[0], Value::Null);
        assert_eq!(query.1[1], Value::Null);
    }

    #[test]
    fn test_join() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let user_table = Table::new("users", db.clone())
            .with_alias("u")
            .with_field("name")
            .with_field("role_id");
        let role_table = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("role_description");

        let table = user_table.with_join(role_table, "role_id");

        let query = table.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT u.name, u.role_id, r.id AS r_id, r.role_description AS r_role_description FROM users AS u LEFT JOIN roles AS r ON (u.role_id = r.id)"
        );
    }

    #[test]
    fn join_table_with_joins() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let person = Table::new("person", db.clone())
            .with_field("id")
            .with_field("name")
            .with_field("parent_id");

        let father = person.clone().with_alias("father");
        let grandfather = person.clone().with_alias("grandfather");

        let person = person.with_join(father.with_join(grandfather, "parent_id"), "parent_id");

        let query = person.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT p.id, p.name, p.parent_id, \
            father.id AS father_id, father.name AS father_name, father.parent_id AS father_parent_id, \
            grandfather.id AS grandfather_id, grandfather.name AS grandfather_name, grandfather.parent_id AS grandfather_parent_id \
            FROM person AS p \
            LEFT JOIN person AS father ON (p.parent_id = father.id) \
            LEFT JOIN person AS grandfather ON (father.parent_id = grandfather.id)"
        );
    }

    #[test]
    fn test_condition_on_joined_table_field() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut user_table = Table::new("users", db.clone())
            .with_field("name")
            .with_field("role_id");
        let role_table = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("role_type");

        let join = user_table.add_join(role_table, "role_id");

        user_table.add_condition(join.get_field("role_type").unwrap().eq(&json!("admin")));

        let query = user_table.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT u.name, u.role_id, r.id AS r_id, r.role_type AS r_role_type FROM users AS u LEFT JOIN roles AS r ON (u.role_id = r.id) WHERE (r.role_type = {})"
        );
        assert_eq!(query.1[0], json!("admin"));
    }

    #[test]
    fn test_conditions_moved_into_on() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut user_table = Table::new("users", db.clone())
            .with_field("name")
            .with_field("role_id");
        let mut role_table = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("role_type");

        role_table.add_condition(
            role_table
                .get_field("role_type")
                .unwrap()
                .eq(&json!("admin")),
        );

        user_table.add_join(role_table, "role_id");

        let query = user_table.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT u.name, u.role_id, r.id AS r_id, r.role_type AS r_role_type FROM users AS u LEFT JOIN roles AS r ON (u.role_id = r.id) AND (r.role_type = {})"
        );
        assert_eq!(query.1[0], json!("admin"));
    }

    #[test]
    fn test_nested_conditions_moved_into_on() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut user_table = Table::new("users", db.clone())
            .with_field("name")
            .with_field("role_id");
        let mut role_table = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("role_type");

        role_table.add_condition(Condition::or(
            role_table
                .get_field("role_type")
                .unwrap()
                .eq(&json!("admin")),
            role_table
                .get_field("role_type")
                .unwrap()
                .eq(&json!("writer")),
        ));

        user_table.add_join(role_table, "role_id");

        let query = user_table.get_select_query().render_chunk().split();

        // TODO: due to Condition::or() implementation, it renders second argument
        // into expression. In fact we push our luck here - perhaps the field we
        // are recursively changing is not even of our table.
        //
        // Ideally table alias should be set before a bunch of Fields are given away
        assert_eq!(
            query.0,
            "SELECT u.name, u.role_id, r.id AS r_id, r.role_type AS r_role_type FROM users AS u \
            LEFT JOIN roles AS r ON (u.role_id = r.id) AND \
            ((r.role_type = {}) OR (role_type = {}))"
        );
        assert_eq!(query.1[0], json!("admin"));
    }

    #[test]
    #[should_panic]
    fn test_join_panic() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let user_table = Table::new("users", db.clone()).with_alias("u");
        let role_table = Table::new("roles", db.clone()).with_alias("u");

        // will panic, both tables want "u" alias
        user_table.with_join(role_table, "role_id");
    }

    #[test]
    fn test_field_query() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut roles = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("name");

        roles.add_condition(roles.get_field("name").unwrap().eq(&"admin".to_string()));
        let query = roles.field_query(roles.get_field("id").unwrap());

        assert_eq!(
            query.render_chunk().sql().clone(),
            "SELECT id FROM roles WHERE (name = {})".to_owned()
        );

        let mut users = Table::new("users", db.clone())
            .with_field("id")
            .with_field("role_id");

        users.add_condition(users.get_field("role_id").unwrap().in_expr(&query));
        let query = users.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, role_id FROM users WHERE (role_id IN (SELECT id FROM roles WHERE (name = {})))"
        );
        assert_eq!(query.1[0], json!("admin"));
    }

    #[test]
    fn test_add_ref() {
        struct UserSet {}
        impl UserSet {
            fn table() -> Table<MockDataSource> {
                let data = json!([]);
                let db = MockDataSource::new(&data);
                let mut table = Table::new("users", db)
                    .with_field("id")
                    .with_field("name")
                    .with_field("role_id");

                table.add_ref("role", |u| {
                    let mut r = RoleSet::table();
                    r.add_condition(
                        r.get_field("id")
                            .unwrap()
                            // .eq(u.get_field("role_id").unwrap()),
                            .in_expr(&u.field_query(u.get_field("role_id").unwrap())),
                    );
                    r
                });
                table
            }
        }

        struct RoleSet {}
        impl RoleSet {
            fn table() -> Table<MockDataSource> {
                let data = json!([]);
                let db = MockDataSource::new(&data);
                Table::new("roles", db)
                    .with_field("id")
                    .with_field("role_type")
            }
        }

        let mut user_table = UserSet::table();

        user_table.add_condition(user_table.get_field("id").unwrap().eq(&123));
        let user_roles = user_table.get_ref("role").unwrap();

        let query = user_roles.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, role_type FROM roles WHERE (id IN (SELECT role_id FROM users WHERE (id = {})))"
        );
    }

    #[test]
    fn test_father_child() {
        struct PersonSet {}
        impl PersonSet {
            fn table() -> Table<MockDataSource> {
                let data = json!([]);
                let db = MockDataSource::new(&data);
                let table = Table::new("persons", db)
                    .with_field("id")
                    .with_field("name")
                    .with_field("parent_id")
                    .has_one("parent", "parent_id", || PersonSet::table())
                    .has_many("children", "parent_id", || PersonSet::table());

                table
            }
        }

        let mut john = PersonSet::table();
        john.add_condition(john.get_field("name").unwrap().eq(&"John".to_string()));

        let children = john.get_ref("children").unwrap();

        let query = children.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, name, parent_id FROM persons WHERE (parent_id IN (SELECT id FROM persons WHERE (name = {})))"
        );

        let grand_children = john
            .get_ref("children")
            .unwrap()
            .get_ref("children")
            .unwrap();

        let query = grand_children.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, name, parent_id FROM persons WHERE \
            (parent_id IN (SELECT id FROM persons WHERE \
            (parent_id IN (SELECT id FROM persons WHERE (name = {})\
            ))\
            ))"
        );

        let parent_name = john
            .get_ref("parent")
            .unwrap()
            .field_query(john.get_field("name").unwrap());

        let query = parent_name.render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT name FROM persons WHERE (id IN (SELECT parent_id FROM persons WHERE (name = {})))"
        );
    }

    #[test]
    fn test_expression_query() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut orders = Table::new("orders", db.clone())
            .with_field("price")
            .with_field("qty");

        orders.add_expression_before_query("total", |t| {
            expr_arc!(
                "{}*{}",
                t.get_field("price").unwrap().clone(),
                t.get_field("qty").unwrap().clone()
            )
            .render_chunk()
        });

        let query = orders.get_select_query().render_chunk().split();

        assert_eq!(query.0, "SELECT price, qty FROM orders");

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
        struct ItemLine {
            price: f64,
            qty: i32,
            total: f64,
        }

        let query = orders
            .get_select_query_for_struct(ItemLine::default())
            .render_chunk()
            .split();
        assert_eq!(
            query.0,
            "SELECT price, qty, (price*qty) AS total FROM orders"
        );
    }
}
