use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

use crate::condition::Condition;
use crate::field::Field;
use crate::lazy_expression::LazyExpression;
use crate::prelude::Operations;
use crate::table::Table;
use crate::traits::column::Column;
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;
use crate::traits::sql_chunk::SqlChunk;

impl<T: DataSource, E: Entity> Table<T, E> {
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
    pub fn search_for_field(&self, field_name: &str) -> Option<Box<dyn Column>> {
        // perhaps we have a field like this
        // let field = self.get_field(field_name);

        if let Some(field) = self.get_field(field_name) {
            return Some(Box::new(field));
        }

        // maybe joined table have a field we want
        for (_, join) in self.joins.iter() {
            if let Some(field) = join.table().get_field(field_name) {
                return Some(Box::new(field));
            }
        }

        // maybe we have a lazy expression
        if let Some(lazy_expression) = self.lazy_expressions.get(field_name) {
            return match lazy_expression {
                LazyExpression::AfterQuery(_) => None,
                LazyExpression::BeforeQuery(expr) => {
                    let x = (expr)(self);
                    Some(Box::new(x))
                }
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        mocks::datasource::MockDataSource,
        prelude::{Operations, SqlChunk},
        table::Table,
    };

    #[test]
    fn test_field_query() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut roles = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("name");

        roles.add_condition(roles.get_field("name").unwrap().eq(&"admin".to_string()));
        let query = roles.field_query(roles.get_field("id").unwrap());

        assert_eq!(
            query.render_chunk().sql().clone(),
            "SELECT id FROM roles WHERE (name = {})".to_owned()
        );

        let mut users = Table::new("users", db.clone())
            .with_field("id")
            .with_field("role_id");

        users.add_condition(users.get_field("role_id").unwrap().in_expr(&query));
        let query = users.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, role_id FROM users WHERE (role_id IN (SELECT id FROM roles WHERE (name = {})))"
        );
        assert_eq!(query.1[0], json!("admin"));
    }
}
