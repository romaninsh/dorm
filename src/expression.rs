use crate::traits::{column::Column, renderable::Renderable};

#[macro_export]
macro_rules! expr {
    ($fmt:expr $(, $arg:expr)*) => {{
        Expression::new(
            $fmt,
            vec![
                $( $arg.to_string(), )*
            ]
        )
    }}
}
pub struct Expression {
    expression: String,
    parameters: Vec<String>,
}

impl Expression {
    pub fn new(expression: &str, parameters: Vec<String>) -> Expression {
        Expression {
            expression: expression.to_string(),
            parameters: parameters.iter().map(|p| p.to_string()).collect(),
        }
    }
}

impl Renderable for Expression {
    fn render(&self) -> String {
        let mut result = self.expression.clone();
        for param in &self.parameters {
            // This is a simple placeholder replacement that assumes the '{}' placeholders are in the order of parameters.
            // It's a naive implementation and should be improved for real use.
            result = result.replacen("{}", &format!("'{}'", param.replace("'", "''")), 1);
        }
        format!("({})", result)
    }
}

impl Column for Expression {
    fn render_column(&self, alias: &str) -> String {
        format!("({}) AS {}", self.render(), alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let expression = expr!("{} + {}", "3", "5");
        assert_eq!(expression.render(), "('3' + '5')");
    }

    #[test]
    fn test_sql_quoting() {
        let expression = expr!("Hello {}", "O'Reilly");
        assert_eq!(expression.render(), "(Hello 'O''Reilly')");
    }

    #[test]
    fn test_multiple_replacements() {
        let expression = expr!("{} - {} = {}", "10", "5", "5");
        assert_eq!(expression.render(), "('10' - '5' = '5')");
    }
}
