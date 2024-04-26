use crate::expression::Expression;
use crate::traits::sql_chunk::SqlChunk;

#[derive(Debug)]
pub struct Condition<'a> {
    field: &'static str,
    operation: &'static str,
    value: Box<dyn SqlChunk<'a>>,
}

impl<'a> Condition<'a> {
    pub fn new(
        field: &'static str,
        operation: &'static str,
        value: Box<dyn SqlChunk<'a>>,
    ) -> Condition<'a> {
        Condition {
            field,
            operation,
            value,
        }
    }
}

impl<'a> SqlChunk<'a> for Condition<'a> {
    fn render_chunk(&self) -> crate::traits::sql_chunk::PreRender {
        let pr = self.value.as_ref().render_chunk();
        Expression::new(format!("{} {} {{}}", self.field, self.operation), vec![&pr]).render_chunk()
    }
}

impl<'a> SqlChunk<'a> for &Condition<'a> {
    fn render_chunk(&self) -> crate::traits::sql_chunk::PreRender {
        let pr = self.value.as_ref().render_chunk();
        Expression::new(format!("{} {} {{}}", self.field, self.operation), vec![&pr]).render_chunk()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_condition() {
//         let field = "id";

//         let condition = Condition {
//             field: &field,
//             operation: "=",
//             value: "1".to_string(),
//         };

//         assert_eq!(condition.render(), "id = '1'");
//     }
// }
