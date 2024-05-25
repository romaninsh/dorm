use crate::expression::Expression;

use super::sql_chunk::SqlChunk;

pub trait Column: SqlChunk {
    fn render_column(&self, alias: Option<&str>) -> Expression;
    fn calculated(&self) -> bool;
}
