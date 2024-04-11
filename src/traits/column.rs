use crate::traits::renderable::Renderable;

pub trait Column: Renderable {
    fn render_column(&self, alias: &str) -> String;
}
