use anyhow::{anyhow, Result};

use crate::prelude::Operations;
use crate::reference::Reference;
use crate::table::Table;
use crate::traits::datasource::DataSource;

impl<T: DataSource> Table<T> {
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

    pub fn add_ref(
        &mut self,
        relation: &str,
        cb: impl Fn(&Table<T>) -> Table<T> + 'static + Sync + Send,
    ) {
        let reference = Reference::new(cb);
        self.refs.insert(relation.to_string(), reference);
    }

    pub fn get_ref(&self, field: &str) -> Result<Table<T>> {
        Ok(self
            .refs
            .get(field)
            .ok_or_else(|| anyhow!("Reference not found"))?
            .table(self))
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
}
