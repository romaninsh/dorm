use std::ptr::eq;
use std::sync::{Arc, Mutex};

use crate::condition::Condition;
use crate::expr_arc;
use crate::expression::ExpressionArc;
use crate::field::Field;
use crate::join::Join;
use crate::lazy_expression::LazyExpression;
use crate::prelude::{AssociatedQuery, Expression, JoinQuery, Operations};
use crate::query::{JoinType, Query, QueryConditions, QueryType};
use crate::reference::Reference;
use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;
use crate::traits::sql_chunk::SqlChunk;
use crate::uniqid::UniqueIdVendor;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::{Map, Value};

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
    fields: IndexMap<String, Field>,
    joins: IndexMap<String, Arc<Join<T>>>,
    lazy_expressions: IndexMap<String, LazyExpression<T>>,
    refs: IndexMap<String, Reference<T>>,

    table_aliases: Arc<Mutex<UniqueIdVendor>>,
}

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
            field.set_table_alias(alias.to_string());
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

    /// Adds a new field to the table. Note, that Field may use an alias
    fn add_field(&mut self, field_name: String, field: Field) {
        self.fields.insert(field_name, field);
    }

    /// Returns a field by name
    pub fn get_field(&self, field: &str) -> Option<&Field> {
        self.fields.get(field)
    }

    /// Handy way to access fields
    pub fn fields(&self) -> &IndexMap<String, Field> {
        &self.fields
    }

    /// When building a table - a simple way to add a typical field. For a
    /// more sophisticated way use `add_field`
    pub fn with_field(mut self, field: &str) -> Self {
        self.add_field(
            field.to_string(),
            Field::new(field.to_string(), self.table_alias.clone()),
        );
        self
    }

    /// Adds a field that is also a title field. Title field will be
    /// used in the UI to represent the record.
    pub fn with_title_field(mut self, field: &str) -> Self {
        self.title_field = Some(field.to_string());
        self.with_field(field)
    }

    pub fn with_id_field(mut self, field: &str) -> Self {
        self.id_field = Some(field.to_string());
        self.with_field(field)
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
        Ok(self.with_condition(Condition::from_field(field, op, Arc::new(Box::new(value)))))
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

    pub fn add_ref(
        &mut self,
        relation: &str,
        cb: impl Fn(&Table<T>) -> Table<T> + 'static + Sync + Send,
    ) {
        let reference = Reference::new(cb);
        self.refs.insert(relation.to_string(), reference);
    }

    pub fn has_many(
        mut self,
        relation: &str,
        foreign_key: &str,
        cb: impl Fn() -> Table<T> + 'static + Sync + Send,
    ) -> Self {
        let foreign_key = foreign_key.to_string();
        self.add_ref(relation, move |p| {
            let mut c = cb();
            let foreign_field = c
                .get_field(&foreign_key)
                .unwrap_or_else(|| panic!("Field '{}' not found", foreign_key));
            let id_field = p
                .get_field("id")
                .unwrap_or_else(|| panic!("Field 'id' not found"));

            c.add_condition(foreign_field.in_expr(&p.field_query(id_field)));
            c
        });
        self
    }

    pub fn has_one(
        mut self,
        relation: &str,
        foreign_key: &str,
        cb: impl Fn() -> Table<T> + 'static + Sync + Send,
    ) -> Self {
        let foreign_key = foreign_key.to_string();
        self.add_ref(relation, move |p| {
            let mut c = cb();
            let id_field = c
                .get_field("id")
                .unwrap_or_else(|| panic!("Field 'id' not found"));
            let foreign_field = p
                .get_field(&foreign_key)
                .unwrap_or_else(|| panic!("Field '{}' not found", foreign_key));

            c.add_condition(id_field.in_expr(&p.field_query(foreign_field)));
            c
        });
        self
    }

    pub fn get_ref(&self, field: &str) -> Result<Table<T>> {
        Ok(self
            .refs
            .get(field)
            .ok_or_else(|| anyhow!("Reference not found"))?
            .table(self))
    }

    pub fn id(&self) -> Field {
        // Field::new("test".to_string(), Some("test".to_string()))
        self.fields.get("id").unwrap().clone()
    }
    pub fn with_id(self, id: Value) -> Self {
        let f = self.id().eq(&id);
        self.with_condition(f)
    }

    /// Left-Joins their_table table and return self. Assuming their_table has set id field,
    /// but we still have to specify foreign key in our own table. For more complex
    /// joins use `join_table` method.
    pub fn with_join(mut self, their_table: Table<T>, our_foreign_id: &str) -> Self {
        self.add_join(their_table, our_foreign_id);
        self
    }

    pub fn add_join(&mut self, mut their_table: Table<T>, our_foreign_id: &str) -> Arc<Join<T>> {
        // before joining, make sure there are no alias clashes
        if eq(&*self.table_aliases, &*their_table.table_aliases) {
            panic!(
                "Tables are already joined: {}, {}",
                self.table_name, their_table.table_name
            )
        }

        if their_table
            .table_aliases
            .lock()
            .unwrap()
            .has_conflict(&self.table_aliases.lock().unwrap())
        {
            panic!(
                "Table alias conflict while joining: {}, {}",
                self.table_name, their_table.table_name
            )
        }

        self.table_aliases
            .lock()
            .unwrap()
            .merge(their_table.table_aliases.lock().unwrap().to_owned());

        // Get information about their_table
        let their_table_name = their_table.table_name.clone();
        if their_table.table_alias.is_none() {
            let their_table_alias = self
                .table_aliases
                .lock()
                .unwrap()
                .get_one_of_uniq_id(UniqueIdVendor::all_prefixes(&their_table_name));
            their_table.set_alias(&their_table_alias);
        };
        let their_table_id = their_table.id();

        // Give alias to our table as well
        if self.table_alias.is_none() {
            let our_table_alias = self
                .table_aliases
                .lock()
                .unwrap()
                .get_one_of_uniq_id(UniqueIdVendor::all_prefixes(&self.table_name));
            self.set_alias(&our_table_alias);
        }
        let their_table_alias = their_table.table_alias.as_ref().unwrap().clone();

        let mut on_condition = QueryConditions::on().add_condition(
            self.get_field(our_foreign_id)
                .unwrap()
                .eq(&their_table_id)
                .render_chunk(),
        );

        // Any condition in their_table should be moved into ON condition
        for condition in their_table.conditions.iter() {
            on_condition = on_condition.add_condition(condition.render_chunk());
        }
        their_table.conditions = Vec::new();

        // Create a join
        let join = JoinQuery::new(
            JoinType::Left,
            crate::query::QuerySource::Table(their_table_name, Some(their_table_alias.clone())),
            on_condition,
        );
        self.joins.insert(
            their_table_alias.clone(),
            Arc::new(Join::new(their_table, join)),
        );

        self.get_join(&their_table_alias).unwrap()
    }

    pub fn get_join(&self, table_alias: &str) -> Option<Arc<Join<T>>> {
        self.joins.get(table_alias).map(|r| r.clone())
    }

    pub fn get_empty_query(&self) -> Query {
        let mut query = Query::new().set_table(&self.table_name, None);
        for condition in self.conditions.iter() {
            query = query.add_condition(condition.clone());
        }
        query
    }

    // TODO: debug why this overwrites the previous fields
    fn add_fields_into_query(&self, mut query: Query, alias_prefix: Option<&str>) -> Query {
        for (field_key, field_val) in &self.fields {
            let field_val = if let Some(alias_prefix) = &alias_prefix {
                let alias = format!("{}_{}", alias_prefix, field_key);
                let mut field_val = field_val.clone();
                field_val.set_field_alias(alias);
                field_val
            } else {
                field_val.clone()
            };
            query = query.add_column(
                field_val
                    .get_field_alias()
                    .unwrap_or_else(|| field_key.clone()),
                field_val,
            );
        }

        for (alias, join) in &self.joins {
            query = query.add_join(join.join_query().clone());
            query = join.add_fields_into_query(query, Some(alias));
        }

        query
    }

    pub fn get_select_query(&self) -> Query {
        let mut query = Query::new().set_table(&self.table_name, self.table_alias.clone());
        query = self.add_fields_into_query(query, None);
        for condition in self.conditions.iter() {
            query = query.add_condition(condition.clone());
        }
        query
    }

    pub fn get_insert_query(&self) -> Query {
        let mut query = Query::new()
            .set_table(&self.table_name, None)
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

    pub fn field_query(&self, field: &Field) -> AssociatedQuery<T> {
        let query = self
            .get_empty_query()
            .add_column(field.name(), field.clone());
        AssociatedQuery::new(query, self.data_source.clone())
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
        self.table().clone().with_condition(condition)
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
        // assert_eq!(query.0, "SELECT price, qty, price*qty AS total FROM orders");
    }
}
