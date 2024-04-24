use std::ops::Deref;

pub struct Condition {
    field: &'static str,
    operation: &'static str,
    value: String,
}

impl Condition {
    pub fn new(field: &'static str, operation: &'static str, value: String) -> Condition {
        Condition {
            field,
            operation,
            value,
        }
    }
}

impl Deref for Condition {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}

// impl<'a> Renderable<'a> for Condition {
//     fn render(&self) -> String {
//         format!("{} {} '{}'", self.field, self.operation, self.value)
//     }

//     fn params(&self) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> {
//         vec![]
//     }
// }

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
