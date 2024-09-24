use anyhow::Result;
use std::any::Any;
use std::sync::Arc;

use crate::condition::Condition;
use crate::field::Field;
use crate::prelude::AssociatedQuery;
use crate::table::Table;

use super::datasource::DataSource;
use super::entity::Entity;

/// When defining references between tables, AnyTable represents
/// a target table, that can potentially be associated with a
/// different data source.
///
/// The implication is that reference fields need to be explicitly
/// fetched and resulting set of "id"s can then be used to define
/// related queries.
///
/// Table::has_unrelated() can be used to define relation like this.
/// To traverse the relation, use Table::get_unrelated_ref("relation") or
/// Table::get_unrelated_ref_as<D: DataSource, E: Entity>(ds, "relation").
///
/// If tables are defined in the same data source, use has_one(),
/// has_many(), which rely on RelatedTable trait.
///
pub trait AnyTable: Any + Send + Sync {
    fn as_any(self) -> Box<dyn Any>;

    fn as_any_ref(&self) -> &dyn Any;

    fn get_field(&self, name: &str) -> Option<&Arc<Field>>;

    fn add_condition(&mut self, condition: Condition);
}

/// When defining references between tables, RelatedTable represents
/// a target table, that resides in the same DataSource and
/// therefore can be referenced inside a query without explicitly
/// fetching the "id"s.
///
///
pub trait RelatedTable<T: DataSource>: AnyTable {
    fn field_query(&self, field: Arc<Field>) -> AssociatedQuery<T>;
}
