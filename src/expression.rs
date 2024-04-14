use crate::traits::{column::Column, renderable::Renderable};

#[macro_export]
macro_rules! expr {
    ($fmt:expr $(, $arg:expr)*) => {{
        Expression::new(
            $fmt,
            vec![
                $( $arg, )*
            ]
        )
    }}
}
pub struct Expression<'a> {
    expression: &'a str,
    parameters: Vec<&'a dyn Renderable<'a>>,
}

impl<'a> Expression<'a> {
    pub fn new(expression: &'a str, parameters: Vec<&'a dyn Renderable<'a>>) -> Expression<'a> {
        Expression {
            expression,
            parameters,
        }
    }
}

impl<'a> Renderable<'a> for Expression<'a> {
    fn render(&self) -> String {
        let mut result = self.expression.to_string();
        for param in &self.parameters {
            // This is a simple placeholder replacement that assumes the '{}' placeholders are in the order of parameters.
            // It's a naive implementation and should be improved for real use.
            result = result.replacen("{}", &format!("{}", param.render()), 1);
        }
        format!("({})", result)
    }
}

impl<'a> Column<'a> for Expression<'a> {
    fn render_column(&self, alias: &str) -> String {
        format!("({}) AS {}", self.render(), alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let a = "3".to_owned();
        let b = "5".to_owned();
        let expression = expr!("{} + {}", &a, &b);
        assert_eq!(expression.render(), "('3' + '5')");
    }

    #[test]
    fn test_sql_quoting() {
        let name = "O'Reilly".to_owned();
        let expression = expr!("Hello {}", &name);
        assert_eq!(expression.render(), "(Hello 'O''Reilly')");
    }

    #[test]
    fn test_multiple_replacements() {
        let a = "10".to_owned();
        let b = "5".to_owned();
        let c = "5".to_owned();
        let expression = expr!("{} - {} = {}", &a, &b, &c);
        assert_eq!(expression.render(), "('10' - '5' = '5')");
    }
}
