use crate::sql::Expression;

use super::super::sql::chunk::Chunk;

pub trait Column: Chunk {
    fn render_column(&self, alias: Option<&str>) -> Expression;
    fn calculated(&self) -> bool;
}
