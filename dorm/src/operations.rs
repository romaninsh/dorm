use std::sync::Arc;

use crate::{
    condition::Condition,
    expr_arc,
    expression::{Expression, ExpressionArc},
    traits::sql_chunk::SqlChunk,
};

pub trait Operations: SqlChunk {
    // fn in_vec(&self, other: Vec<impl SqlChunk>) -> Condition {
    //     Condition::from_expression(
    //         self.render_chunk(),
    //         "IN",
    //         Arc::new(Box::new(ExpressionArc::from_vec(
    //             other.into_iter().map(|x| x.render_chunk()).collect(),
    //             ", ",
    //         ))),
    //     )
    // }
    fn in_expr(&self, other: &impl SqlChunk) -> Condition {
        Condition::from_expression(
            self.render_chunk(),
            "IN",
            Arc::new(Box::new(expr_arc!("({})", other.render_chunk()))),
        )
    }
    fn eq(&self, other: &impl SqlChunk) -> Condition {
        Condition::from_expression(
            self.render_chunk(),
            "=",
            Arc::new(Box::new(other.render_chunk())),
        )
    }

    fn ne(&self, other: impl SqlChunk) -> Condition {
        Condition::from_expression(
            self.render_chunk(),
            "!=",
            Arc::new(Box::new(other.render_chunk())),
        )
    }

    fn gt(&self, other: impl SqlChunk) -> Condition {
        Condition::from_expression(
            self.render_chunk(),
            ">",
            Arc::new(Box::new(other.render_chunk())),
        )
    }

    fn lt(&self, other: impl SqlChunk) -> Condition {
        Condition::from_expression(
            self.render_chunk(),
            "<",
            Arc::new(Box::new(other.render_chunk())),
        )
    }

    /*
    fn gt(&self, other: impl SqlChunk) -> Expression {
        expr_arc!("({}) > ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }

    fn lt(&self, other: impl SqlChunk) -> Expression {
        expr_arc!("({}) < ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }
    */

    fn add(&self, other: impl SqlChunk) -> Expression {
        expr_arc!("({}) + ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }

    fn sub(&self, other: impl SqlChunk) -> Expression {
        expr_arc!("({}) - ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }

    fn concat(arg: Vec<Arc<Box<dyn SqlChunk>>>) -> Expression {
        ExpressionArc::from_vec(arg, ", ").render_chunk()
    }

    fn upper(&self) -> Expression {
        expr_arc!("UPPER({})", self.render_chunk()).render_chunk()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{mocks::datasource::MockDataSource, prelude::*};

    #[test]
    fn test_upper() {
        let a = Arc::new(Field::new("name".to_string(), None));
        let b = a.upper();

        assert_eq!(b.render_chunk().sql(), "UPPER(name)");
    }

    #[test]
    fn test_upper_in_table() {
        let data = json!([]);
        let t = Table::new("product", MockDataSource::new(&data))
            .with_field("name")
            .with_expression("name_caps", |t| t.get_field("name").unwrap().upper());

        let query = t
            .get_select_query_for_field_names(&["name", "name_caps"])
            .render_chunk();
        assert_eq!(
            query.sql(),
            "SELECT name, (UPPER(name)) AS name_caps FROM product"
        );
    }
}
