use crate::{expression::Expression, field::Field, traits::renderable::Renderable};

pub enum QueryType {
    Select,
}

pub struct Query<'a> {
    table: String,
    query_type: QueryType,
    columns: Vec<Box<dyn Renderable<'a> + 'a>>,
    conditions: Vec<&'a dyn Renderable<'a>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Query<'a> {
    pub fn new(table: &str) -> Query {
        Query {
            table: table.to_string(),
            query_type: QueryType::Select,
            columns: Vec::new(),
            conditions: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn set_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }

    pub fn add_column(mut self, field: Box<dyn Renderable<'a> + 'a>) -> Self {
        self.columns.push(field);
        self
    }

    pub fn add_condition(mut self, cond: &'a dyn Renderable<'a>) -> Self {
        self.conditions.push(cond);
        self
    }

    // Simplified ways to define a field with a string
    pub fn add_column_field(self, field: &str) -> Self {
        self.add_column(Box::new(Field::new(field)))
    }

    pub fn add_column_expr(self, expression: Expression<'a>) -> Self {
        self.add_column(Box::new(expression))
    }
}

impl<'a> Renderable<'a> for Query<'a> {
    fn render(&self) -> String {
        let fields = self
            .columns
            .iter()
            .map(|f| f.render())
            .collect::<Vec<String>>()
            .join(", ");
        format!(
            "SELECT {} FROM {}{}",
            fields,
            self.table,
            if self.conditions.is_empty() {
                "".to_string()
            } else {
                format!(
                    " WHERE {}",
                    self.conditions
                        .iter()
                        .map(|c| c.render())
                        .collect::<Vec<String>>()
                        .join(" AND ")
                )
            }
        )
    }
}
