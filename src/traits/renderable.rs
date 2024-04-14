use async_trait::async_trait;

#[async_trait]
pub trait Renderable {
    fn render(&self) -> String;
}
