use crate::expression::Expression;

use super::sql_chunk::SqlChunk;

pub trait Column: SqlChunk {
    fn render_column(&self, alias: &str) -> Expression;
    fn calculated(&self) -> bool;
}
