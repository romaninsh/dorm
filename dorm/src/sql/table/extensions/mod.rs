//! Table extensions
//!
//! Table extensions are a way to add additional functionality to a table. They
//! are implemented as a trait, and can be added to a table using the
//! [`Table::with_extension()`] method.

use std::sync::Arc;

use crate::{prelude::Entity, sql::Query, traits::datasource::DataSource};

use super::{AnyTable, Table};

trait TableExtension {
    fn init(&self, _table: Arc<Box<dyn AnyTable>>) {}
    fn before_select_query(&self, _table: Arc<Box<dyn AnyTable>>, query: Query) -> Query {
        query
    }
    fn before_delete_query(&self, _table: Arc<Box<dyn AnyTable>>, query: Query) -> Query {
        query
    }
}

struct Hooks {
    hooks: Vec<Box<dyn TableExtension>>,
}
impl Hooks {
    pub fn new() -> Self {
        Hooks { hooks: vec![] }
    }
    /// Add new hook to the table
    pub fn add_hook(&mut self, table: Arc<Box<dyn AnyTable>>, hook: Box<dyn TableExtension>) {
        hook.init(table);
        self.hooks.push(hook);
    }

    pub fn before_select_query(&self, table: Arc<Box<dyn AnyTable>>, mut query: Query) -> Query {
        for hook in self.hooks.iter() {
            query = hook.before_select_query(table.clone(), query);
        }
        query
    }
}

mod soft_delete;

use indexmap::IndexMap;
pub use soft_delete::SoftDelete;
