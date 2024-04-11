use crate::traits::renderable::Renderable;

pub struct Field {
    name: String,
    alias: Option<String>,
}

impl Field {
    pub fn new(name: &str) -> Field {
        Field {
            name: name.to_string(),
            alias: None,
        }
    }

    pub fn alias(&mut self, alias: &str) -> &mut Field {
        self.alias = Some(alias.to_string());
        self
    }
}

impl Renderable for Field {
    fn render(&self) -> String {
        match &self.alias {
            Some(alias) => format!("{} AS {}", self.name, alias),
            None => self.name.clone(),
        }
    }
}
