//! Table extensions
//!
//! Table extensions are a way to add additional functionality to a table. They
//! are implemented as a trait, and can be added to a table using the
//! [`Table::with_extension()`] method.

use anyhow::Result;
use std::sync::Arc;

use crate::{prelude::Entity, sql::Query, traits::datasource::DataSource};

use super::{SqlTable, Table};

trait TableExtension {
    fn init(&self) {}
    fn before_select_query(&self, _table: &mut dyn SqlTable, query: &mut Query) -> Result<()> {
        Ok(())
    }
    fn before_delete_query(&self, _table: Arc<Box<dyn SqlTable>>, query: &mut Query) -> Result<()> {
        Ok(())
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
    pub fn add_hook(&mut self, hook: Box<dyn TableExtension>) {
        hook.init();
        self.hooks.push(hook);
    }

    pub fn before_select_query(&self, table: &mut dyn SqlTable, query: &mut Query) -> Result<()> {
        for hook in self.hooks.iter() {
            hook.before_select_query(table, query);
        }
        Ok(())
    }
}

mod soft_delete;

use anyhow::Ok;
use indexmap::IndexMap;
pub use soft_delete::SoftDelete;
