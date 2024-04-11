use crate::traits::column::Column;
use crate::traits::renderable::Renderable;

pub struct Field {
    name: String,
}

impl Field {
    pub fn new(name: &str) -> Field {
        Field {
            name: name.to_string(),
        }
    }
}

impl Renderable for Field {
    fn render(&self) -> String {
        self.name.clone()
    }
}

impl Column for Field {
    fn render_column(&self, alias: &str) -> String {
        format!("{} AS {}", self.render(), alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field() {
        let field = Field::new("id");
        assert_eq!(field.render(), "id");
    }

    #[test]
    fn test_field_with_alias() {
        let field = Field::new("id");
        assert_eq!(field.render_column("user_id"), "id AS user_id");
    }
}
