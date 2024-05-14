use std::sync::Arc;

use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::{json, Value};

use crate::{
    expr, expr_arc,
    expression::{Expression, ExpressionArc},
    field::Field,
    traits::{column::Column, sql_chunk::SqlChunk},
};

mod parts;

pub use parts::*;

#[derive(Debug)]
pub struct Query {
    table: QuerySource,
    query_type: QueryType,
    columns: IndexMap<String, Arc<Box<dyn Column>>>,
    conditions: Vec<Arc<Box<dyn SqlChunk>>>,

    where_conditions: QueryConditions,
    having_conditions: QueryConditions,
    joins: Vec<JoinQuery>,
}

#[derive(Debug)]
pub enum UniqAlias {
    FieldAlias,
    TableAlias,
}

impl Query {
    pub fn new() -> Query {
        Query {
            table: QuerySource::None,
            query_type: QueryType::Select,
            columns: IndexMap::new(),
            conditions: Vec::new(),
            where_conditions: QueryConditions::where_(),
            having_conditions: QueryConditions::having(),
            joins: Vec::new(),
        }
    }

    pub fn set_table(mut self, table: &str) -> Self {
        self.table = QuerySource::Table(table.to_string(), None);
        self
    }

    pub fn set_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }

    pub fn add_column(self, name: String, field: impl Column + 'static) -> Self {
        self.add_column_arc(name, Arc::new(Box::new(field)))
    }

    pub fn add_column_arc(mut self, name: String, field: Arc<Box<dyn Column>>) -> Self {
        self.columns.insert(name, field);
        self
    }

    pub fn add_where_condition(mut self, cond: Expression) -> Self {
        self.where_conditions = self.where_conditions.add_condition(cond);
        self
    }

    pub fn add_having_condition(mut self, cond: Expression) -> Self {
        self.having_conditions = self.having_conditions.add_condition(cond);
        self
    }

    pub fn add_join(mut self, join: JoinQuery) -> Self {
        self.joins.push(join);
        self
    }

    pub fn add_condition(self, cond: impl SqlChunk + 'static) -> Self {
        self.add_condition_arc(Arc::new(Box::new(cond)))
    }

    pub fn add_condition_arc(mut self, cond: Arc<Box<dyn SqlChunk>>) -> Self {
        self.conditions.push(cond);
        self
    }

    // Simplified ways to define a field with a string
    pub fn add_column_field(self, name: &str) -> Self {
        self.add_column(name.to_string(), Field::new(name.to_string(), None))
    }

    fn render_where(&self) -> Expression {
        if self.conditions.is_empty() {
            Expression::empty()
        } else {
            let conditions = ExpressionArc::from_vec(self.conditions.clone(), " AND ");
            expr_arc!(" WHERE {}", conditions).render_chunk()
        }
    }

    fn render_select(&self) -> Result<Expression> {
        let fields = Expression::from_vec(
            self.columns
                .iter()
                .map(|f| f.1.render_column(f.0).render_chunk())
                .collect(),
            ", ",
        );

        Ok(expr_arc!(
            "SELECT {} {}{}{}",
            fields,
            self.table.render_chunk(),
            Expression::from_vec(self.joins.iter().map(|x| x.render_chunk()).collect(), ""),
            self.render_where()
        )
        .render_chunk())
    }

    fn render_insert(&self) -> Result<Expression> {
        let QuerySource::Table(table, _) = self.table.clone() else {
            return Err(anyhow!("Call set_table() for insert query"));
        };

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
            .map(|_| json!(None as Option<Value>))
            .collect::<Vec<Value>>();

        Ok(expr_arc!(
            format!(
                "{} INTO {} ({}) VALUES ({{}}) returning id",
                match self.query_type {
                    QueryType::Insert => "INSERT",
                    QueryType::Replace => "REPLACE",
                    _ => panic!("Invalid query type"),
                },
                table,
                fields
            ),
            Expression::new(values_str, values)
        )
        .render_chunk())
    }

    fn render_delete(&self) -> Result<Expression> {
        let QuerySource::Table(table, _) = self.table.clone() else {
            return Err(anyhow!("Call set_table() for insert query"));
        };

        Ok(expr_arc!(format!("DELETE FROM {}{{}}", table), self.render_where()).render_chunk())
    }

    pub fn preview(&self) -> String {
        self.render_chunk().preview()
    }
}

impl SqlChunk for Query {
    fn render_chunk(&self) -> Expression {
        match &self.query_type {
            QueryType::Select => self.render_select(),
            QueryType::Insert | QueryType::Replace => self.render_insert(),
            QueryType::Delete => self.render_delete(),
            QueryType::Expression(expr) => Ok(expr.clone()),
        }
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;

    use super::*;

    #[test]
    fn test_where() {
        let expr1 = expr!("name = {}", "John");
        let expr2 = expr!("age > {}", 30);

        let query = Query::new()
            .set_table("users")
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
        let (sql, params) = Query::new()
            .set_table("users")
            .add_column_field("id")
            .add_column_field("name")
            .add_column("calc".to_string(), expr_arc!("1 + 1"))
            .render_chunk()
            .split();

        assert_eq!(sql, "SELECT id, name, (1 + 1) AS calc FROM users");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_insert() {
        let (sql, params) = Query::new()
            .set_table("users")
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

    #[test]
    fn test_expression() {
        let (sql, params) = Query::new()
            .set_type(QueryType::Expression(expr!("CALL some_procedure()")))
            .render_chunk()
            .split();

        assert_eq!(sql, "CALL some_procedure()");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_join_query() {
        let query = Query::new()
            .set_table("users")
            .add_column_field("id")
            .add_column_field("name");

        let join = JoinQuery::new(
            JoinType::Left,
            QuerySource::Table("roles".to_string(), None),
            QueryConditions::on().add_condition(expr!("users.role_id = roles.id")),
        );

        let (sql, params) = query.add_join(join).render_chunk().split();

        assert_eq!(
            sql,
            "SELECT id, name FROM users LEFT JOIN roles ON users.role_id = roles.id"
        );
        assert_eq!(params.len(), 0);
    }
}
