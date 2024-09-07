use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

use crate::condition::Condition;
use crate::field::Field;
use crate::prelude::Operations;
use crate::table::Table;
use crate::traits::datasource::DataSource;
use crate::traits::sql_chunk::SqlChunk;

impl<T: DataSource> Table<T> {
    /// Adds a new field to the table. Note, that Field may use an alias
    pub fn add_field(&mut self, field_name: String, field: Field) {
        self.fields.insert(field_name, Arc::new(field));
    }

    /// Returns a field reference by name.
    pub fn get_field(&self, field: &str) -> Option<Arc<Field>> {
        self.fields.get(field).map(|f| f.clone())
    }

    /// Handy way to access fields
    pub fn fields(&self) -> &IndexMap<String, Arc<Field>> {
        &self.fields
    }

    /// When building a table - a simple way to add a typical field. For a
    /// more sophisticated way use `add_field`
    pub fn with_field(mut self, field: &str) -> Self {
        self.add_field(
            field.to_string(),
            Field::new(field.to_string(), self.table_alias.clone()),
        );
        self
    }

    /// Adds a field that is also a title field. Title field will be
    /// used in the UI to represent the record.
    pub fn with_title_field(mut self, field: &str) -> Self {
        self.title_field = Some(field.to_string());
        self.with_field(field)
    }

    pub fn with_id_field(mut self, field: &str) -> Self {
        self.id_field = Some(field.to_string());
        self.with_field(field)
    }

    pub fn add_condition_on_field(
        self,
        field: &'static str,
        op: &'static str,
        value: impl SqlChunk + 'static,
    ) -> Result<Self> {
        let field = self
            .get_field(field)
            .ok_or_else(|| anyhow!("Field not found: {}", field))?
            .clone();
        Ok(self.with_condition(Condition::from_field(field, op, Arc::new(Box::new(value)))))
    }

    pub fn id(&self) -> Arc<Field> {
        let id_field = if self.id_field.is_some() {
            let x = self.id_field.clone().unwrap();
            x.clone()
        } else {
            "id".to_string()
        };
        self.get_field(&id_field).unwrap()
    }
    pub fn with_id(self, id: Value) -> Self {
        let f = self.id().eq(&id);
        self.with_condition(f)
    }
}
