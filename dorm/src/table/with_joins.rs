use std::ptr::eq;
use std::sync::Arc;

use crate::join::Join;
use crate::prelude::{Operations, SqlChunk};
use crate::query::{JoinQuery, JoinType, QueryConditions};
use crate::table::Table;
use crate::traits::datasource::DataSource;
use crate::uniqid::UniqueIdVendor;

impl<T: DataSource> Table<T> {
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
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use serde_json::json;

    use super::*;
    use crate::{
        condition::Condition,
        mocks::datasource::MockDataSource,
        prelude::{Operations, SqlChunk},
    };
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
}
