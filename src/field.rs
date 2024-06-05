use std::ops::Deref;
use std::sync::Arc;

use crate::condition::Condition;
use crate::expr;
use crate::expression::expression_arc::WrapArc;
use crate::expression::Expression;
use crate::operations::Operations;
use crate::traits::column::Column;
use crate::traits::sql_chunk::SqlChunk;

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    table_alias: Option<String>,
    field_alias: Option<String>,
}

impl Field {
    pub fn new(name: String, table_alias: Option<String>) -> Field {
        Field {
            name,
            table_alias,
            field_alias: None,
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    fn name_with_table(&self) -> String {
        match &self.table_alias {
            Some(table_alias) => format!("{}.{}", table_alias, self.name),
            None => self.name.clone(),
        }
    }
    pub fn set_table_alias(&mut self, alias: String) {
        self.table_alias = Some(alias);
    }
    pub fn set_field_alias(&mut self, alias: String) {
        self.field_alias = Some(alias);
    }

    pub fn get_field_alias(&self) -> Option<String> {
        self.field_alias.clone()
    }
}

impl Operations for Arc<Field> {
    fn eq(&self, other: &impl SqlChunk) -> Condition {
        Condition::from_field(self.clone(), "=", WrapArc::wrap_arc(other.render_chunk()))
    }

    // fn add(&self, other: impl SqlChunk) -> Expression {
    //     let chunk = other.render_chunk();
    //     expr_arc!(format!("{} + {{}}", &self.name), chunk).render_chunk()
    // }
}

impl SqlChunk for Arc<Field> {
    fn render_chunk(&self) -> Expression {
        Expression::new(format!("{}", self.name_with_table()), vec![])
    }
}

impl Column for Arc<Field> {
    fn render_column(&self, mut alias: Option<&str>) -> Expression {
        // If the alias is the same as the field name, we don't need to render it
        if alias.is_some() && alias.unwrap() == self.name {
            alias = None;
        }

        let alias = alias.or(self.field_alias.as_deref());

        if let Some(alias) = alias {
            expr!(format!("{} AS {}", self.name_with_table(), alias))
        } else {
            expr!(format!("{}", self.name_with_table()))
        }
        .render_chunk()
    }
    fn calculated(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field() {
        let field = Arc::new(Field::new("id".to_string(), None));
        let (sql, params) = field.render_chunk().split();

        assert_eq!(sql, "id");
        assert_eq!(params.len(), 0);

        let (sql, params) = field.render_column(Some("id")).render_chunk().split();
        assert_eq!(sql, "id");
        assert_eq!(params.len(), 0);

        let (sql, params) = &field.render_column(Some("id_alias")).render_chunk().split();
        assert_eq!(sql, "id AS id_alias");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_eq() {
        let field = Arc::new(Field::new("id".to_string(), None));
        let (sql, params) = field.eq(&1).render_chunk().split();

        assert_eq!(sql, "(id = {})");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], 1);

        let f_age = Arc::new(Field::new("age".to_string(), Some("u".to_string())));
        let (sql, params) = f_age.add(5).eq(&18).render_chunk().split();

        assert_eq!(sql, "((u.age) + ({}) = {})");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], 5);
        assert_eq!(params[1], 18);
    }
}
