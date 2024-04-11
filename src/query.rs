#[allow(dead_code)]
use crate::{field::Field, traits::renderable::Renderable};

pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

pub struct Query {
    table: String,
    query_type: QueryType,
    fields: Vec<Field>,
}

impl Query {
    pub fn new(table: &str) -> Query {
        Query {
            table: table.to_string(),
            query_type: QueryType::Select,
            fields: Vec::new(),
        }
    }

    pub fn set_type(&mut self, query_type: QueryType) -> &mut Query {
        self.query_type = query_type;
        self
    }

    pub fn fields(&mut self, fields: Vec<&str>) -> &mut Query {
        self.fields = fields.iter().map(|f| Field::new(f)).collect();
        self
    }

    pub fn field(&mut self, field: &str) -> &mut Query {
        self.fields.push(Field::new(field));
        self
    }
}

impl Renderable for Query {
    fn render(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|f| f.render())
            .collect::<Vec<String>>()
            .join(", ");
        format!("SELECT {} FROM {}", fields, self.table)
    }
}
