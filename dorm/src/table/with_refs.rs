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
