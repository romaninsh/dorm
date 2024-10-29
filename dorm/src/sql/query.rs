use std::sync::Arc;

use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use serde_json::Value;

use crate::{
    expr_arc,
    field::Field,
    sql::chunk::Chunk,
    sql::expression::{Expression, ExpressionArc},
    traits::column::Column,
};

mod parts;

pub use parts::*;

#[derive(Debug, Clone)]
pub struct Query {
    table: QuerySource,
    with: IndexMap<String, QuerySource>,
    distinct: bool,
    query_type: QueryType,
    columns: IndexMap<String, Arc<Box<dyn Column>>>,
    set_fields: IndexMap<String, Value>,

    where_conditions: QueryConditions,
    having_conditions: QueryConditions,
    joins: Vec<JoinQuery>,

    group_by: Vec<Expression>,
    order_by: Vec<Expression>,
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
            with: IndexMap::new(),
            distinct: false,
            query_type: QueryType::Select,
            columns: IndexMap::new(),

            set_fields: IndexMap::new(),

            where_conditions: QueryConditions::where_(),
            having_conditions: QueryConditions::having(),
            joins: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
        }
    }

    pub fn is_distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn with_table(mut self, table: &str, alias: Option<String>) -> Self {
        self.table = QuerySource::Table(table.to_string(), alias);
        self
    }

    pub fn with_with(mut self, alias: &str, subquery: Query) -> Self {
        self.with.insert(
            alias.to_string(),
            QuerySource::Query(Arc::new(Box::new(subquery)), None),
        );
        self
    }

    pub fn with_source(mut self, source: QuerySource) -> Self {
        self.table = source;
        self
    }

    pub fn with_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }

    pub fn without_columns(mut self) -> Self {
        self.columns = IndexMap::new();
        self
    }

    pub fn with_column(self, name: String, field: impl Column + 'static) -> Self {
        self.with_column_arc(name, Arc::new(Box::new(field)))
    }

    pub fn with_column_arc(mut self, name: String, field: Arc<Box<dyn Column>>) -> Self {
        self.columns.insert(name, field);
        self
    }

    pub fn with_where_condition(mut self, cond: Expression) -> Self {
        self.where_conditions = self.where_conditions.add_condition(cond);
        self
    }

    pub fn with_having_condition(mut self, cond: Expression) -> Self {
        self.having_conditions = self.having_conditions.add_condition(cond);
        self
    }

    // pub fn with_condition(mut self, cond: Condition) -> Self {
    //     self.where_conditions = self.where_conditions.add_condition(cond);
    // }

    pub fn with_join(mut self, join: JoinQuery) -> Self {
        self.joins.push(join);
        self
    }

    pub fn with_condition(self, cond: impl Chunk + 'static) -> Self {
        self.with_where_condition(cond.render_chunk())
    }

    pub fn with_condition_arc(self, cond: Arc<Box<dyn Chunk>>) -> Self {
        self.with_where_condition(cond.render_chunk())
    }

    pub fn with_group_by(mut self, group_by: Expression) -> Self {
        self.group_by.push(group_by);
        self
    }

    pub fn with_order_by(mut self, order_by: Expression) -> Self {
        self.order_by.push(order_by);
        self
    }

    // Simplified ways to define a field with a string
    pub fn with_column_field(self, name: &str) -> Self {
        self.with_column(
            name.to_string(),
            Arc::new(Field::new(name.to_string(), None)),
        )
    }

    pub fn with_set_field(mut self, field: &str, value: Value) -> Self {
        match self.query_type {
            QueryType::Insert | QueryType::Update | QueryType::Replace => {
                self.set_fields.insert(field.to_string(), value);
                self
            }
            _ => {
                panic!("Call set_set_field() for insert query");
            }
        }
    }

    fn render_with(&self) -> Expression {
        if self.with.is_empty() {
            Expression::empty()
        } else {
            let with = self
                .with
                .iter()
                .map(|(name, query)| {
                    expr_arc!(format!("{} AS {{}}", name), query.render_prefix("")).render_chunk()
                })
                .collect::<Vec<Expression>>();
            let e = Expression::from_vec(with, ", ");
            expr_arc!("WITH {} ", e).render_chunk()
        }
    }

    fn render_group_by(&self) -> Expression {
        if self.group_by.is_empty() {
            Expression::empty()
        } else {
            let group_by = Expression::from_vec(self.group_by.clone(), ", ");
            expr_arc!(" GROUP BY {}", group_by).render_chunk()
        }
    }

    fn render_order_by(&self) -> Expression {
        if self.order_by.is_empty() {
            Expression::empty()
        } else {
            let mut rev_vec = self.order_by.clone();
            rev_vec.reverse();
            let order_by = Expression::from_vec(rev_vec, ", ");
            expr_arc!(" ORDER BY {}", order_by).render_chunk()
        }
    }

    fn render_select(&self) -> Result<Expression> {
        let fields = Expression::from_vec(
            self.columns
                .iter()
                .map(|f| f.1.render_column(Some(f.0)).render_chunk())
                .collect(),
            ", ",
        );

        Ok(expr_arc!(
            format!(
                "{{}}SELECT{} {{}} {{}}{{}}{{}}{{}}{{}}{{}}",
                if self.distinct { " DISTINCT" } else { "" }
            ),
            self.render_with(),
            fields,
            self.table.render_chunk(),
            Expression::from_vec(self.joins.iter().map(|x| x.render_chunk()).collect(), ""),
            self.where_conditions.render_chunk(),
            self.render_group_by(),
            self.render_order_by(),
            self.having_conditions.render_chunk()
        )
        .render_chunk())
    }

    fn render_insert(&self) -> Result<Expression> {
        let QuerySource::Table(table, _) = self.table.clone() else {
            return Err(anyhow!("Call set_table() for insert query"));
        };

        let fields = self
            .set_fields
            .iter()
            .map(|(k, _)| k.clone())
            .collect::<Vec<String>>()
            .join(", ");

        let values_str = self
            .set_fields
            .iter()
            .map(|_| "{}".to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let values = self
            .set_fields
            .iter()
            .map(|(_, f)| f.clone())
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

    fn render_update(&self) -> Result<Expression> {
        let QuerySource::Table(table, _) = self.table.clone() else {
            return Err(anyhow!("Call set_table() for insert query"));
        };

        let set_fields = self
            .set_fields
            .iter()
            .map(|(k, v)| {
                let expr = expr_arc!(format!("{} = {{}}", k), v.clone());
                let boxed_chunk: Box<dyn Chunk> = Box::new(expr);
                Arc::new(boxed_chunk)
            })
            .collect::<Vec<Arc<Box<dyn Chunk>>>>();

        let set_fields = ExpressionArc::from_vec(set_fields, ", ");

        Ok(expr_arc!(
            format!("UPDATE {} SET {{}}{{}}", table),
            set_fields,
            self.where_conditions.render_chunk()
        )
        .render_chunk())
    }

    fn render_delete(&self) -> Result<Expression> {
        let QuerySource::Table(table, _) = self.table.clone() else {
            return Err(anyhow!("Call set_table() for insert query"));
        };

        Ok(expr_arc!(
            format!("DELETE FROM {}{{}}", table),
            self.where_conditions.render_chunk()
        )
        .render_chunk())
    }

    pub fn preview(&self) -> String {
        self.render_chunk().preview()
    }
}

impl Chunk for Query {
    fn render_chunk(&self) -> Expression {
        match &self.query_type {
            QueryType::Select => self.render_select(),
            QueryType::Insert | QueryType::Replace => self.render_insert(),
            QueryType::Update => self.render_update(),
            QueryType::Delete => self.render_delete(),
            QueryType::Expression(expr) => Ok(expr.clone()),
        }
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_where() {
        let expr1 = expr!("name = {}", "John");
        let expr2 = expr!("age > {}", 30);

        let query = Query::new()
            .with_table("users", None)
            .with_condition(expr1)
            .with_condition(expr2);

        let wher = query.where_conditions.render_chunk();

        let (sql, params) = wher.render_chunk().split();

        assert_eq!(sql, " WHERE name = {} AND age > {}");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], Value::String("John".to_string()));
        assert_eq!(params[1], Value::Number(30.into()));
    }

    #[test]
    fn test_select() {
        let (sql, params) = Query::new()
            .with_table("users", None)
            .with_column_field("id")
            .with_column_field("name")
            .with_column("calc".to_string(), expr_arc!("1 + 1"))
            .render_chunk()
            .split();

        assert_eq!(sql, "SELECT id, name, (1 + 1) AS calc FROM users");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_insert() {
        let (sql, params) = Query::new()
            .with_table("users", None)
            .with_type(QueryType::Insert)
            .with_set_field("name", "John".into())
            .with_set_field("surname", "Doe".into())
            .with_set_field("age", 30.into())
            .render_chunk()
            .split();

        assert_eq!(
            sql,
            "INSERT INTO users (name, surname, age) VALUES ({}, {}, {}) returning id"
        );
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], json!("John"));
        assert_eq!(params[1], json!("Doe"));
        assert_eq!(params[2], json!(30));
    }

    #[test]
    fn test_update() {
        let (sql, params) = Query::new()
            .with_table("users", None)
            .with_type(QueryType::Update)
            .with_set_field("name", "John".into())
            .with_set_field("surname", "Doe".into())
            .with_set_field("age", 30.into())
            .with_condition(expr!("id = {}", 1))
            .render_chunk()
            .split();

        assert_eq!(
            sql,
            "UPDATE users SET name = {}, surname = {}, age = {} WHERE id = {}"
        );
        assert_eq!(params.len(), 4);
        assert_eq!(params[0], json!("John"));
        assert_eq!(params[1], json!("Doe"));
        assert_eq!(params[2], json!(30));
        assert_eq!(params[3], json!(1));
    }

    #[test]
    fn test_expression() {
        let (sql, params) = Query::new()
            .with_type(QueryType::Expression(expr!("CALL some_procedure()")))
            .render_chunk()
            .split();

        assert_eq!(sql, "CALL some_procedure()");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_expression_field() {
        let (sql, params) = Query::new()
            .with_table("product", None)
            .with_column("name_caps".to_string(), expr!("UPPER(name)"))
            .render_chunk()
            .split();

        assert_eq!(sql, "SELECT (UPPER(name)) AS name_caps FROM product");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_join_query() {
        let query = Query::new()
            .with_table("users", None)
            .with_column_field("id")
            .with_column_field("name");

        let join = JoinQuery::new(
            JoinType::Left,
            QuerySource::Table("roles".to_string(), None),
            QueryConditions::on().add_condition(expr!("users.role_id = roles.id")),
        );

        let (sql, params) = query.with_join(join).render_chunk().split();

        assert_eq!(
            sql,
            "SELECT id, name FROM users LEFT JOIN roles ON users.role_id = roles.id"
        );
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_render_with() {
        let roles = Query::new()
            .with_table("roles", None)
            .with_column_field("id")
            .with_column_field("role_name");

        let outer_query = Query::new()
            .with_table("users", None)
            .with_with("roles", roles)
            .with_join(JoinQuery::new(
                JoinType::Inner,
                QuerySource::Table("roles".to_string(), None),
                QueryConditions::on().add_condition(expr!("users.role_id = roles.id")),
            ))
            .with_column_field("user_name")
            .with_column_field("roles.role_name");

        let (sql, params) = outer_query.render_chunk().split();

        assert_eq!(sql, "WITH roles AS (SELECT id, role_name FROM roles) SELECT user_name, roles.role_name FROM users JOIN roles ON users.role_id = roles.id");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_group_and_order() {
        let query = Query::new()
            .with_table("users", None)
            .with_column_field("id")
            .with_column_field("name")
            .with_column_field("age")
            .with_group_by(expr!("name"))
            .with_order_by(expr!("age DESC"));

        let (sql, params) = query.render_chunk().split();

        assert_eq!(
            sql,
            "SELECT id, name, age FROM users GROUP BY name ORDER BY age DESC"
        );
        assert_eq!(params.len(), 0);
    }
}
