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
}

pub fn concat(arg: Vec<Arc<Box<dyn SqlChunk>>>) -> Expression {
    ExpressionArc::from_vec(arg, ", ").render_chunk()
}
