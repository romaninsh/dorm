use crate::{
    operations::Operations,
    traits::{
        column::Column,
        sql_chunk::{PreRender, SqlChunk},
    },
};

#[macro_export]
macro_rules! expr {
    ($fmt:expr $(, $arg:expr)*) => {{
        Expression::new(
            $fmt.to_string(),
            vec![
                $( &$arg as &dyn crate::traits::sql_chunk::SqlChunk, )*
            ]
        )
    }}
}

#[derive(Debug)]
pub struct Expression<'a> {
    expression: String,
    parameters: Vec<&'a dyn SqlChunk<'a>>,
}

impl<'a> Expression<'a> {
    pub fn new(expression: String, parameters: Vec<&'a dyn SqlChunk<'a>>) -> Expression<'a> {
        Expression {
            expression,
            parameters,
        }
    }
}

impl<'a> SqlChunk<'a> for Expression<'a> {
    fn render_chunk(&self) -> PreRender {
        let mut param_iter = self.parameters.iter();
        let mut sql = self.expression.to_string().clone();

        let mut param_out = Vec::new();

        let token = "{}";

        while let Some(index) = sql.find(token) {
            if let Some(param) = param_iter.next() {
                let (param_sql, param_values) = param.render_chunk().split();

                sql.replace_range(index..index + token.len(), &param_sql);
                param_out.extend(param_values);
            } else {
                break;
            }
        }

        PreRender::new((sql, param_out))
    }
}

impl<'a> Column<'a> for Expression<'a> {
    fn render_column(&self, alias: &str) -> PreRender {
        let expression = format!("({}) AS `{}`", self.expression, alias);

        Expression::new(expression, self.parameters.clone()).render_chunk()
    }
    fn calculated(&self) -> bool {
        true
    }
}

impl<'a> Operations<'a> for Expression<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_expression() {
        let expression = Expression::new("Hello World".to_string(), vec![]);
        let (sql, params) = expression.render_chunk().split();

        assert_eq!(sql, "Hello World");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_expr_without_parameters() {
        let expression = expr!("Hello World");
        let (sql, params) = expression.render_chunk().split();

        assert_eq!(sql, "Hello World");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_nested_expr_without_parameters() {
        let nested = expr!("Nested");
        let expression = expr!("Hello {} World", nested);
        let (sql, params) = expression.render_chunk().split();

        assert_eq!(sql, "Hello Nested World");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_nested_expr_without_parameters_lifetimes() {
        let a = String::from("Nested");

        let expression = Expression::new("{} World".to_string(), vec![&a]);
        let nested = Expression::new("Hello {}".to_string(), vec![&expression]);
        let (sql, params) = nested.render_chunk().split();

        assert_eq!(sql, "Hello World");
        assert_eq!(params.len(), 0);

        drop(nested);

        let nested = Expression::new("Hello {}".to_string(), vec![&expression]);
        let (sql, params) = nested.render_chunk().split();

        assert_eq!(sql, "Hello World");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_nested_expression() {
        let nested = Expression::new("Nested".to_string(), vec![]);
        let expression =
            Expression::new("Hello {} World".to_string(), vec![&nested as &dyn SqlChunk]);

        let (sql, params) = expression.render_chunk().split();

        assert_eq!(sql, "Hello Nested World");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_multiple_replacements() {
        let a = json!(10);
        let b = json!(5);
        let c = json!(5);
        let expression = expr!("{} - {} = {}", a, b, c);
        let (sql, params) = expression.render_chunk().split();

        assert_eq!(sql, "{} - {} = {}");
        assert_eq!(params.len(), 3);
        assert_eq!(params, vec![json!(10), json!(5), json!(5)]);
    }

    #[test]
    fn test_nested_expr() {
        let a = "10".to_owned();
        let b = "5";
        let c = 4;

        let expr2 = expr!("{} + {}", b, c);
        let expr1 = expr!("{} + {}", a, expr2);

        let (sql, params) = expr1.render_chunk().split();

        assert_eq!(sql, "{} + {} + {}");
        assert_eq!(params.len(), 3);
        assert_eq!(params, vec![json!("10"), json!("5"), json!(4)]);
    }

    #[test]
    fn test_column() {
        let a = "10".to_owned();
        let b = "5";
        let c = 4;

        let expr2 = expr!("{} + {}", b, c);
        let expr1 = expr!("{} + {}", a, expr2);

        let column = expr1.render_column("result");
        let (sql, params) = column.split();

        assert_eq!(sql, "({} + {} + {}) AS `result`");
        assert_eq!(params.len(), 3);
        assert_eq!(params, vec![json!("10"), json!("5"), json!(4)]);
    }

    #[test]
    fn test_lifetimes() {
        let expr2 = Expression::new("Hello".to_string(), vec![]);
        {
            let expr1 = Expression::new("{}".to_string(), vec![&expr2]);
            drop(expr1);
        }

        let test = expr2;
    }
}
