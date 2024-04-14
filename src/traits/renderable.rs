pub trait Renderable<'a> {
    fn render(&self) -> String;
}

impl Renderable<'_> for String {
    fn render(&self) -> String {
        format!("'{}'", self.clone().replace("'", "''"))
    }
}

// Does not work because sizing
// impl Renderable<'_> for str {
//     fn render(&self) -> String {
//         self.to_string()
//     }
// }
