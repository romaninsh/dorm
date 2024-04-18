use crate::{expression::Expression, field::Field, traits::renderable::Renderable};

pub enum QueryType {
    Select,
    Insert,
    Delete,
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

    fn render_where(&self) -> String {
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
    }

    fn render_select(&self) -> String {
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
            self.render_where()
        )
    }

    fn render_insert(&self) -> String {
        let fields = self
            .columns
            .iter()
            .map(|f| f.render())
            .collect::<Vec<String>>()
            .join(", ");
        let placeholders = (1..=self.columns.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<String>>()
            .join(", ");
        format!(
            "INSERT INTO {} ({}) VALUES ({}) returning id",
            self.table, fields, placeholders
        )
    }

    fn render_delete(&self) -> String {
        format!("DELETE FROM {}{}", self.table, self.render_where())
    }
}

impl<'a> Renderable<'a> for Query<'a> {
    fn render(&self) -> String {
        match self.query_type {
            QueryType::Select => self.render_select(),
            QueryType::Insert => self.render_insert(),
            QueryType::Delete => self.render_delete(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let query = Query::new("users")
            .add_column_field("id")
            .add_column_field("name")
            .add_column_expr(Expression::new("1 + 1", vec![]))
            .render();

        assert_eq!(query, "SELECT id, name, (1 + 1) FROM users");
    }

    #[test]
    fn test_insert() {
        let query = Query::new("users")
            .set_type(QueryType::Insert)
            .add_column_field("name")
            .add_column_field("surname")
            .add_column_field("age")
            .render();

        assert_eq!(
            query,
            "INSERT INTO users (name, surname, age) VALUES ($1, $2, $3) returning id"
        );
    }
}
