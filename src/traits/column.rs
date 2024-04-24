use super::sql_chunk::{PreRender, SqlChunk};

pub trait Column<'a>: SqlChunk<'a> {
    fn render_column(&self, alias: &str) -> PreRender;
    fn calculated(&self) -> bool;
}
