use serde_json::Value;

use crate::{operations::Operations, traits::sql_chunk::SqlChunk};

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
}

impl Operations for Expression {}
