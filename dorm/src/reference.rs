use core::fmt;
use std::sync::Arc;

use crate::{prelude::Table, traits::datasource::DataSource};

#[derive(Clone)]
pub struct Reference<T: DataSource> {
    // table: Option<Table<T>>,
    get_table: Arc<Box<dyn Fn(&Table<T>) -> Table<T> + 'static + Sync + Send>>,
}

impl<T: DataSource> Reference<T> {
    pub fn new<F>(get_table: F) -> Reference<T>
    where
        F: Fn(&Table<T>) -> Table<T> + 'static + Sync + Send,
    {
        Reference {
            // table: None,
            get_table: Arc::new(Box::new(get_table)),
        }
    }
    pub fn table(&self, table: &Table<T>) -> Table<T> {
        (self.get_table)(table)
        // if self.table.is_none() {
        //     self.table = Some((self.get_table)(table));
        // }
        // self.table.as_ref().unwrap()
    }
}

impl<T: DataSource + fmt::Debug> fmt::Debug for Reference<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Reference")
            // Cannot print `get_table` since it's a function pointer
            .finish()
    }
}
