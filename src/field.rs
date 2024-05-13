use crate::condition::Condition;
use crate::expression::expression_arc::WrapArc;
use crate::expression::{Expression, ExpressionArc};
use crate::operations::Operations;
use crate::traits::column::Column;
use crate::traits::sql_chunk::SqlChunk;
use crate::{expr, expr_arc};

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    table_alias: Option<String>,
}

impl Field {
    pub fn new(name: String, table_alias: Option<String>) -> Field {
        Field { name, table_alias }
    }
    fn name_with_table(&self) -> String {
        match &self.table_alias {
            Some(table_alias) => format!("{}.{}", table_alias, self.name),
            None => self.name.clone(),
        }
    }
}

impl Operations for Field {
    fn eq(&self, other: impl SqlChunk) -> Condition {
        Condition::from_field(self.clone(), "=", WrapArc::wrap_arc(other.render_chunk()))
    }

    // fn add(&self, other: impl SqlChunk) -> Expression {
    //     let chunk = other.render_chunk();
    //     expr_arc!(format!("{} + {{}}", &self.name), chunk).render_chunk()
    // }
}

impl SqlChunk for Field {
    fn render_chunk(&self) -> Expression {
        Expression::new(format!("{}", self.name), vec![])
    }
}

impl Column for Field {
    fn render_column(&self, alias: &str) -> Expression {
        (if self.name == alias {
            expr!(format!("{}", self.name_with_table()))
        } else {
            expr!(format!("{} AS {}", self.name_with_table(), alias))
        })
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
        let field = Field::new("id".to_string(), None);
        let (sql, params) = field.render_chunk().split();

        assert_eq!(sql, "id");
        assert_eq!(params.len(), 0);

        let (sql, params) = field.render_column("id").render_chunk().split();
        assert_eq!(sql, "id");
        assert_eq!(params.len(), 0);

        let (sql, params) = &field.render_column("id_alias").render_chunk().split();
        assert_eq!(sql, "id AS id_alias");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_eq() {
        let field = Field::new("id".to_string(), None);
        let (sql, params) = field.eq(1).render_chunk().split();

        assert_eq!(sql, "id = {}");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], 1);

        let f_age = Field::new("age".to_string(), Some("u".to_string()));
        let (sql, params) = f_age.add(5).eq(18).render_chunk().split();

        assert_eq!(sql, "(age) + ({}) = {}");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], 5);
        assert_eq!(params[1], 18);
    }
}
