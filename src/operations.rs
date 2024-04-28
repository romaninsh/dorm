use std::sync::Arc;

use crate::{
    expr_arc,
    expression::{Expression, ExpressionArc},
    traits::sql_chunk::SqlChunk,
};

pub trait Operations: SqlChunk {
    fn eq(&self, other: impl SqlChunk) -> Expression {
        expr_arc!("({}) = ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }

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
