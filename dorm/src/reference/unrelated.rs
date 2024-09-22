use core::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::json;

use crate::prelude::Table;
use crate::traits::any::AnyTable;
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;

type RelatedTableFx<T, E> = dyn Fn(&Table<T, E>) -> Pin<Box<dyn Future<Output = Box<dyn AnyTable>> + Send + Sync>>
    + Send
    + Sync;

#[derive(Clone)]
pub struct UnrelatedReference<T: DataSource, E: Entity> {
    get_table: Arc<Box<RelatedTableFx<T, E>>>,
}

impl<T: DataSource, E: Entity> UnrelatedReference<T, E> {
    pub fn new<F>(get_table: F) -> UnrelatedReference<T, E>
    where
        F: 'static
            + Fn(&Table<T, E>) -> Pin<Box<dyn Future<Output = Box<dyn AnyTable>> + Send + Sync>>
            + Send
            + Sync,
    {
        UnrelatedReference {
            get_table: Arc::new(Box::new(move |table: &Table<T, E>| {
                Box::pin(get_table(table))
            })),
        }
    }

    pub async fn as_table(&self, table: &Table<T, E>) -> Box<dyn AnyTable> {
        (self.get_table)(table).await
    }
}

impl<T: DataSource + fmt::Debug, E: Entity + fmt::Debug> fmt::Debug for UnrelatedReference<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnrelatedReference").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::datasource::MockDataSource;
    use crate::prelude::{Operations, SqlChunk};
    use crate::traits::entity::EmptyEntity;

    // #[tokio::test]
    // async fn test_unrelated_reference() {
    //     let data = json!([]);
    //     let data_source = MockDataSource::new(&data);

    //     let reference = UnrelatedReference::new(|t: &Table<MockDataSource, EmptyEntity>| async {
    //         let mut t = Table::new("cached_orders", data_source.clone())
    //             .with_field("cached_order_id")
    //             .with_field("cached_data");

    //         let user_id = t.field_query("user_id").get_one().await.unwrap();

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
