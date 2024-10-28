use anyhow::{anyhow, Result};

use crate::reference::RelatedReference;
use crate::table::Table;
use crate::traits::any::RelatedTable;
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;

impl<T: DataSource, E: Entity> Table<T, E> {
    pub fn has_many(
        mut self,
        relation: &str,
        foreign_key: &str,
        cb: impl Fn() -> Box<dyn RelatedTable<T>> + 'static + Sync + Send,
    ) -> Self {
        self.add_ref(relation, RelatedReference::new_many(foreign_key, cb));
        self
    }

    pub fn has_one(
        mut self,
        relation: &str,
        foreign_key: &str,
        cb: impl Fn() -> Box<dyn RelatedTable<T>> + 'static + Sync + Send,
    ) -> Self {
        self.add_ref(relation, RelatedReference::new_one(foreign_key, cb));
        self
    }

    pub fn add_ref(&mut self, relation: &str, reference: RelatedReference<T, E>) {
        self.refs.insert(relation.to_string(), reference);
    }

    pub fn get_ref(&self, field: &str) -> Result<Box<dyn RelatedTable<T>>> {
        self.refs
            .get(field)
            .map(|reference| reference.as_table(self))
            .ok_or_else(|| anyhow!("Reference not found"))
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
    use crate::{
        mocks::datasource::MockDataSource,
        prelude::{AnyTable, EmptyEntity, Operations, SqlChunk},
    };
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
