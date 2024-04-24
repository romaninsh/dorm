use indexmap::IndexMap;
use serde_json::{json, Value};

use crate::{
    expr,
    expression::Expression,
    field::Field,
    traits::{
        column::Column,
        sql_chunk::{PreRender, SqlChunk},
    },
};

#[derive(Debug)]
pub enum QueryType {
    Select,
    Insert,
    Delete,
}

#[derive(Debug)]
pub struct Query<'a> {
    table: String,
    query_type: QueryType,
    columns: IndexMap<String, Box<dyn Column<'a> + 'a>>,
    conditions: Vec<Box<dyn SqlChunk<'a> + 'a>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Query<'a> {
    pub fn new(table: &str) -> Query {
        Query {
            table: table.to_string(),
            query_type: QueryType::Select,
            columns: IndexMap::new(),
            conditions: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn set_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }

    pub fn add_column(mut self, name: String, field: Box<dyn Column<'a> + 'a>) -> Self {
        self.columns.insert(name, field);
        self
    }

    pub fn add_condition(mut self, cond: Box<dyn SqlChunk<'a> + 'a>) -> Self {
        self.conditions.push(cond);
        self
    }

    // Simplified ways to define a field with a string
    pub fn add_column_field(self, name: &str) -> Self {
        self.add_column(name.to_string(), Box::new(Field::new(name.to_string())))
    }

    pub fn add_column_expr(self, name: String, expression: Expression<'a>) -> Self {
        self.add_column(name, Box::new(expression))
    }

    fn render_where(&self) -> PreRender {
        if self.conditions.is_empty() {
            PreRender::empty()
        } else {
            let conditions = PreRender::from_vec(
                self.conditions
                    .iter()
                    .map(|c| c.render_chunk())
                    .collect::<Vec<PreRender>>(),
                " AND ",
            );

            expr!(" WHERE {}", conditions).render_chunk()
        }
    }

    fn render_select(&self) -> PreRender {
        let fields = PreRender::from_vec(
            self.columns
                .iter()
                .map(|f| f.1.render_column(f.0).render_chunk())
                .collect(),
            ", ",
        );

        expr!(
            format!("SELECT {{}} FROM `{}`{{}}", self.table),
            fields,
            self.render_where()
        )
        .render_chunk()
    }

    fn render_insert(&self) -> PreRender {
        let fields = self
            .columns
            .iter()
            .filter(|f| !f.1.calculated())
            .map(|f| f.0.clone())
            .collect::<Vec<String>>()
            .join(", ");

        let values_str = self
            .columns
            .iter()
            .filter(|f| !f.1.calculated())
            .map(|_| "{}".to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let values = self
            .columns
            .iter()
            .filter(|f| !f.1.calculated())
            .map(|f| json!(None as Option<Value>))
            .collect::<Vec<Value>>();

        expr!(
            format!(
                "INSERT INTO {} ({}) VALUES ({{}}) returning id",
                self.table, fields
            ),
            Expression::new(
                values_str,
                values.iter().map(|v| v as &dyn SqlChunk).collect()
            )
        )
        .render_chunk()
    }

    fn render_delete(&self) -> PreRender {
        expr!(
            format!("DELETE FROM {}{{}}", self.table),
            self.render_where()
        )
        .render_chunk()
    }
}

impl<'a> SqlChunk<'a> for Query<'a> {
    fn render_chunk(&self) -> PreRender {
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
    fn test_where() {
        let expr1 = Box::new(expr!("name = {}", "John"));
        let expr2 = Box::new(expr!("age > {}", 30));

        let query = Query::new("users")
            .add_condition(expr1)
            .add_condition(expr2);

        let wher = query.render_where();

        let (sql, params) = wher.render_chunk().split();

        assert_eq!(sql, " WHERE name = {} AND age > {}");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], Value::String("John".to_string()));
        assert_eq!(params[1], Value::Number(30.into()));
    }

    #[test]
    fn test_select() {
        let (sql, params) = Query::new("users")
            .add_column_field("id")
            .add_column_field("name")
            .add_column_expr("calc".to_string(), expr!("1 + 1"))
            .render_chunk()
            .split();

        assert_eq!(sql, "SELECT `id`, `name`, (1 + 1) AS `calc` FROM `users`");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_insert() {
        let (sql, params) = Query::new("users")
            .set_type(QueryType::Insert)
            .add_column_field("name")
            .add_column_field("surname")
            .add_column_field("age")
            .render_chunk()
            .split();

        assert_eq!(
            sql,
            "INSERT INTO users (name, surname, age) VALUES ({}, {}, {}) returning id"
        );
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], Value::Null);
        assert_eq!(params[1], Value::Null);
        assert_eq!(params[2], Value::Null);
    }
}
