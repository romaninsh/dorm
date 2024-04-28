use std::sync::Arc;

use crate::expression::{Expression, ExpressionArc};
use crate::traits::sql_chunk::SqlChunk;

#[derive(Debug, Clone)]
pub struct Condition {
    field: String,
    operation: String,
    value: Arc<Box<dyn SqlChunk>>,
}

#[allow(dead_code)]
impl Condition {
    pub fn new(field: &str, operation: &str, value: Arc<Box<dyn SqlChunk>>) -> Condition {
        Condition {
            field: field.to_string(),
            operation: operation.to_string(),
            value,
        }
    }
}

impl SqlChunk for Condition {
    fn render_chunk(&self) -> Expression {
        ExpressionArc::new(
            format!("{} {} {{}}", self.field, self.operation),
            vec![self.value.clone()],
        )
        .render_chunk()
    }
}

// impl SqlChunk for &Condition {
//     fn render_chunk(&self) -> Expression {
//         let pr = self.value.as_ref().render_chunk();
//         Expression::new(format!("{} {} {{}}", self.field, self.operation), vec![&pr]).render_chunk()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition() {
        let field = "id";

        let condition = Condition::new(field, "=", Arc::new(Box::new("1".to_string())));
        let (sql, params) = condition.render_chunk().split();

        assert_eq!(sql, "id = {}");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "1");
    }
}
