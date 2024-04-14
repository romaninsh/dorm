use crate::traits::renderable::Renderable;

pub trait Column<'a>: Renderable<'a> {
    fn render_column(&self, alias: &str) -> String;
}
