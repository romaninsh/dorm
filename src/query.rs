use crate::{expression::Expression, field::Field, traits::renderable::Renderable};

pub enum QueryType {
    Select,
}

pub struct Query {
    table: String,
    query_type: QueryType,
    columns: Vec<Box<dyn Renderable>>,
}

impl Query {
    pub fn new(table: &str) -> Query {
        Query {
            table: table.to_string(),
            query_type: QueryType::Select,
            columns: Vec::new(),
        }
    }

    pub fn set_type(&mut self, query_type: QueryType) -> &mut Query {
        self.query_type = query_type;
        self
    }

    pub fn add_column(&mut self, field: Box<dyn Renderable>) -> &mut Query {
        self.columns.push(field);
        self
    }

    // Simplified ways to define a field with a string
    pub fn add_column_field(&mut self, field: &str) -> &mut Query {
        self.add_column(Box::new(Field::new(field)));
        self
    }

    pub fn add_column_expr(&mut self, expression: Expression) -> &mut Query {
        self.add_column(Box::new(expression));
        self
    }
}

impl Renderable for Query {
    fn render(&self) -> String {
        let fields = self
            .columns
            .iter()
            .map(|f| f.render())
            .collect::<Vec<String>>()
            .join(", ");
        format!("SELECT {} FROM {}", fields, self.table)
    }
}
