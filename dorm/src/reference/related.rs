use core::fmt;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::json;

use crate::prelude::{Operations, Table};
use crate::traits::any::{AnyTable, RelatedTable};
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;

type RelatedTableFx<T, E> =
    dyn Fn(&Table<T, E>) -> Box<dyn RelatedTable<T>> + Send + Sync + 'static;

// Related table operates with two functions. The first is a simple one, that is
// provided by a client inside "has_many" or "has_one" arguments. The function
// will be provided with a Table<T,E> and it would return RelatedTable<T>
//
// Such a table can be converted into Table<T,E2> if E2 is known, which is in the
// entity logic.
//
// RelatedReference will help you by creating a condition on RelatedTable, but
// it requires that RelatedTable implements add_condition() and field_query()
// functions.

#[derive(Clone)]
pub struct RelatedReference<T: DataSource, E: Entity> {
    get_table: Arc<Box<RelatedTableFx<T, E>>>,
}

impl<T: DataSource, E: Entity> RelatedReference<T, E> {
    pub fn new(
        get_table: impl Fn(&Table<T, E>) -> Box<dyn RelatedTable<T>> + Send + Sync + 'static,
    ) -> RelatedReference<T, E> {
        RelatedReference {
            get_table: Arc::new(Box::new(get_table)),
        }
    }

    pub fn new_many(
        foreign_key: &str,
        cb: impl Fn() -> Box<dyn RelatedTable<T>> + 'static + Sync + Send,
    ) -> RelatedReference<T, E> {
        let foreign_key = foreign_key.to_string();
        RelatedReference::new(move |p| {
            let mut c = cb();
            let foreign_field = c
                .get_field(&foreign_key)
                .unwrap_or_else(|| panic!("Field '{}' not found", foreign_key));
            let id_field = p
                .get_field("id")
                .unwrap_or_else(|| panic!("Field 'id' not found"));

            c.add_condition(foreign_field.in_expr(&p.field_query(id_field.clone())));
            c
        })
    }

    pub fn new_one(
        foreign_key: &str,
        cb: impl Fn() -> Box<dyn RelatedTable<T>> + 'static + Sync + Send,
    ) -> RelatedReference<T, E> {
        let foreign_key = foreign_key.to_string();
        RelatedReference::new(move |p| {
            let mut c = cb();
            let foreign_field = c
                .get_field(&foreign_key)
                .unwrap_or_else(|| panic!("Field '{}' not found", foreign_key))
                .clone();
            let id_field = p
                .get_field("id")
                .unwrap_or_else(|| panic!("Field 'id' not found"));

            c.add_condition(id_field.in_expr(&p.field_query(foreign_field)));
            c
        })
    }

    // pub fn has_one(
    //     mut self,
    //     relation: &str,
    //     foreign_key: &str,
    //     cb: impl Fn() -> Box<dyn AnyTable> + 'static + Sync + Send,
    // ) -> Self {
    //     let foreign_key = foreign_key.to_string();
    //     self.add_ref(relation, move |p| {
    //         let mut c = cb();
    //         let id_field = c
    //             .get_field("id")
    //             .unwrap_or_else(|| panic!("Field 'id' not found"));
    //         let foreign_field = p
    //             .get_field(&foreign_key)
    //             .unwrap_or_else(|| panic!("Field '{}' not found", foreign_key));

    //         c.add_condition(id_field.in_expr(&p.field_query(foreign_field)));
    //         c
    //     });
    //     self
    // }

    pub fn as_table(&self, table: &Table<T, E>) -> Box<dyn RelatedTable<T>> {
        (self.get_table)(table)
    }
}

impl<T: DataSource + fmt::Debug, E: Entity + fmt::Debug> fmt::Debug for RelatedReference<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RelatedReference").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::mocks::datasource::MockDataSource;
    // use crate::prelude::{Operations, SqlChunk};
    // use crate::traits::entity::EmptyEntity;

    // #[tokio::test]
    // async fn test_related_reference() {
    //     let data = json!([]);
    //     let data_source = MockDataSource::new(&data);

    //     let reference = RelatedReference::new(|t: &Table<MockDataSource, EmptyEntity>| {
    //         let mut t = Table::new("cached_orders", data_source.clone())
    //             .with_field("cached_order_id")
    //             .with_field("cached_data");

    //         let user_id = t
    //             .field_query(t.get_field("user_id").unwrap())
    //             .get_one()
    //             .await
    //             .unwrap();

    //         t.add_condition(t.get_field("cached_order_id").unwrap().eq(&user_id));

    //         Box::new(t) as Box<dyn AnyTable>
    //     });

    //     let table = reference
    //         .as_table(&Table::new("users", data_source.clone()))
    //         .await;

    //     let result = table.get_all_data().await;

    //     assert_eq!(result.unwrap(), *data_source.data());
    // }
}
