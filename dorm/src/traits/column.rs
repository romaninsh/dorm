use crate::sql::Expression;

use super::super::sql::chunk::SqlChunk;

pub trait Column: SqlChunk {
    fn render_column(&self, alias: Option<&str>) -> Expression;
    fn calculated(&self) -> bool;
}
