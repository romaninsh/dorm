use serde_json::Value;

use crate::{
    operations::Operations,
    traits::{column::Column, sql_chunk::SqlChunk},
};

#[macro_export]
macro_rules! expr {
    ($fmt:expr $(, $arg:expr)*) => {{
        Expression::new(
            $fmt.to_string(),
            vec![
                $( serde_json::json!($arg), )*
            ]
        )
    }}
}

#[derive(Debug, Clone)]
pub struct Expression {
    expression: String,
    parameters: Vec<Value>,
}

impl SqlChunk for Expression {
    fn render_chunk(&self) -> Expression {
        self.clone()
    }
}

impl Expression {
    pub fn new(expression: String, parameters: Vec<Value>) -> Self {
        Self {
            expression,
            parameters,
        }
    }

    pub fn as_type(value: Value, as_type: &str) -> Self {
        expr!(format!("{{}}::{}", as_type), value)
    }

    pub fn empty() -> Self {
        Self {
            expression: "".to_owned(),
            parameters: vec![],
        }
    }

    pub fn sql(&self) -> &String {
        &self.expression
    }

    pub fn sql_final(&self) -> String {
        let mut sql_final = self.expression.clone();

        let token = "{}";
        let mut num = 0;
        while let Some(index) = sql_final.find(token) {
            num += 1;
            sql_final.replace_range(index..index + token.len(), &format!("${}", num));
        }
        sql_final
    }

    pub fn params(&self) -> &Vec<Value> {
        &self.parameters
    }

    pub fn from_vec(vec: Vec<Expression>, delimiter: &str) -> Self {
        let expression = vec
            .iter()
            .map(|pre| pre.expression.clone())
            .collect::<Vec<String>>()
            .join(delimiter);

        let parameters = vec
            .into_iter()
            .map(|pre| pre.parameters)
            .flatten()
            .collect::<Vec<Value>>();

        Self {
            expression,
            parameters,
        }
    }

    pub fn split(self) -> (String, Vec<Value>) {
        (self.expression, self.parameters)
    }

    pub fn preview(&self) -> String {
        let mut preview = self.expression.clone();
        for param in &self.parameters {
            preview = preview.replacen("{}", &param.to_string(), 1);
        }
        preview
    }
}

impl Column for Expression {
    fn render_column(&self, alias: Option<&str>) -> Expression {
        let expression = if let Some(alias) = alias {
            format!("({}) AS {}", self.expression, alias)
        } else {
            format!("({})", self.expression)
        };

        Expression::new(expression, self.parameters.clone())
    }
    fn calculated(&self) -> bool {
        true
    }
}
impl Operations for Expression {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_as_type() {
        use super::*;
        use crate::traits::sql_chunk::SqlChunk;
        use serde_json::json;

        let expression = Expression::as_type(json!(1), "int");
        let (sql, params) = expression.render_chunk().split();
        assert_eq!(sql, "{}::int");
        assert_eq!(params, vec![Value::Number(1.into())]);
    }
}
