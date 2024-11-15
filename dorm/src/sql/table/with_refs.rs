use std::{ops::Deref, sync::Arc};

use anyhow::{anyhow, Result};

use super::reference::{many::ReferenceMany, one::ReferenceOne, RelatedSqlTable};
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;
use crate::{prelude::EmptyEntity, sql::table::Table};

use super::SqlTable;

impl<T: DataSource, E: Entity> Table<T, E> {
    pub fn has_many(
        mut self,
        relation: &str,
        foreign_key: &str,
        cb: impl Fn() -> Box<dyn SqlTable> + Send + Sync + 'static,
    ) -> Self {
        self.add_ref(relation, Box::new(ReferenceMany::new(foreign_key, cb)));
        self
    }

    pub fn has_one(
        mut self,
        relation: &str,
        foreign_key: &str,
        cb: impl Fn() -> Box<dyn SqlTable> + Send + Sync + 'static,
    ) -> Self {
        // let client = cb();
        self.add_ref(relation, Box::new(ReferenceOne::new(foreign_key, cb)));

        self
    }

    pub fn add_ref(&mut self, relation: &str, reference: Box<dyn RelatedSqlTable>) {
        self.refs.insert(relation.to_string(), Arc::new(reference));
    }

    pub fn get_ref(&self, field: &str) -> Result<Box<dyn SqlTable>> {
        self.refs
            .get(field)
            .map(|reference| {
                let set = reference.get_related_set(self);
                set
            })
            .ok_or_else(|| anyhow!("Reference not found"))
    }

    pub fn get_ref_with_empty_entity(&self, field: &str) -> Result<Table<T, EmptyEntity>> {
        let t = self.get_ref(field)?;
        let t = Box::new(t.as_any_ref());
        let t = t.downcast_ref::<Table<T, EmptyEntity>>().unwrap().clone();
        Ok(t)
    }

    pub fn get_ref_as<T2: DataSource, E2: Entity>(&self, field: &str) -> Result<Table<T2, E2>> {
        let table = self.get_ref(field)?;
        table
            // TODO: not sure why we can't as_any().downcast() here
            .as_any_ref()
            .downcast_ref::<Table<T2, E2>>()
            .map(|t| t.clone())
            .ok_or_else(|| anyhow!("Failed to downcast to specific table type"))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use serde_json::json;

    use super::*;
    use crate::{mocks::datasource::MockDataSource, prelude::*};
    // #[test]
    // fn test_add_ref() {
    //     struct UserSet {}
    //     impl UserSet {
    //         fn table() -> Table<MockDataSource, EmptyEntity> {
    //             let data = json!([]);
    //             let db = MockDataSource::new(&data);
    //             let mut table = Table::new("users", db)
    //                 .with_field("id")
    //                 .with_field("name")
    //                 .with_field("role_id");

    //             table.add_ref("role", |u| {
    //                 let mut r = RoleSet::table();
    //                 r.add_condition(
    //                     r.get_field("id")
    //                         .unwrap()
    //                         // .eq(u.get_field("role_id").unwrap()),
    //                         .in_expr(&u.field_query(u.get_field("role_id").unwrap())),
    //                 );
    //                 r
    //             });
    //             table
    //         }
    //     }

    //     struct RoleSet {}
    //     impl RoleSet {
    //         fn table() -> Table<MockDataSource, EmptyEntity> {
    //             let data = json!([]);
    //             let db = MockDataSource::new(&data);
    //             Table::new("roles", db)
    //                 .with_field("id")
    //                 .with_field("role_type")
    //         }
    //     }

    //     let mut user_table = UserSet::table();

    //     user_table.add_condition(user_table.get_field("id").unwrap().eq(&123));
    //     let user_roles = user_table.get_ref("role").unwrap();

    //     let query = user_roles.get_select_query().render_chunk().split();

    //     assert_eq!(
    //         query.0,
    //         "SELECT id, role_type FROM roles WHERE (id IN (SELECT role_id FROM users WHERE (id = {})))"
    //     );
    // }

    #[test]
    fn test_father_child() {
        struct PersonSet {}
        impl PersonSet {
            fn table() -> Table<MockDataSource, EmptyEntity> {
                let data = json!([]);
                let db = MockDataSource::new(&data);
                let table = Table::new("persons", db)
                    .with_field("id")
                    .with_field("name")
                    .with_field("parent_id")
                    .has_one("parent", "parent_id", || Box::new(PersonSet::table()))
                    .has_many("children", "parent_id", || Box::new(PersonSet::table()));

                table
            }
        }

        let mut john = PersonSet::table();
        john.add_condition(john.get_field("name").unwrap().eq(&"John".to_string()));

        let children: Table<MockDataSource, EmptyEntity> = john.get_ref_as("children").unwrap();

        let query = children.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, name, parent_id FROM persons WHERE (parent_id IN (SELECT id FROM persons WHERE (name = {})))"
        );

        let grand_children = john
            .get_ref_as::<MockDataSource, EmptyEntity>("children")
            .unwrap()
            .get_ref_as::<MockDataSource, EmptyEntity>("children")
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
            .get_ref_with_empty_entity("parent")
            .unwrap()
            .field_query(john.get_field("name").unwrap());

        let query = parent_name.render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT name FROM persons WHERE (id IN (SELECT parent_id FROM persons WHERE (name = {})))"
        );
    }
}
