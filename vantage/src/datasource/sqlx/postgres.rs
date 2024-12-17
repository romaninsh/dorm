use std::sync::Arc;

use anyhow::Result;
use serde_json::{Map, Value};
use sqlx::postgres::PgArguments;
use sqlx::{Column, Row};

use crate::{
    prelude::Query,
    sql::{Chunk, Expression},
    traits::DataSource,
};

use super::sql_to_json::row_to_json;

#[derive(Debug, Clone)]
pub struct Postgres {
    pub pool: Arc<sqlx::PgPool>,
}

impl Postgres {
    pub async fn new(url: &str) -> Self {
        let pool = sqlx::PgPool::connect(url).await.unwrap();
        Self {
            pool: Arc::new(pool),
        }
    }

    // Will be possible extended with some advanced types, that can potentially come out of expression
    pub fn bind<'a>(
        &self,
        mut query: sqlx::query::Query<'a, sqlx::Postgres, PgArguments>,
        expression: &'a Expression,
    ) -> sqlx::query::Query<'a, sqlx::Postgres, PgArguments> {
        for param in expression.params() {
            query = query.bind(param);
        }
        query
    }
}

impl PartialEq for Postgres {
    fn eq(&self, other: &Postgres) -> bool {
        Arc::ptr_eq(&self.pool, &other.pool)
    }
}

impl DataSource for Postgres {
    async fn query_fetch(&self, query: &Query) -> Result<Vec<Map<String, Value>>> {
        let expression = query.render_chunk();
        let sql_final = expression.sql_final();

        let query = sqlx::query(&sql_final);
        let query = self.bind(query, &expression);

        let rows = query.fetch_all(&*self.pool).await?;

        Ok(rows.iter().map(row_to_json).collect())
    }

    async fn query_exec(&self, query: &Query) -> Result<Option<Value>> {
        todo!()
    }

    async fn query_insert(&self, query: &Query, rows: Vec<Vec<Value>>) -> Result<()> {
        todo!()
    }

    async fn query_one(&self, query: &Query) -> Result<Value> {
        let expression = query.render_chunk();
        let sql_final = expression.sql_final();

        let query = sqlx::query(&sql_final);
        let query = self.bind(query, &expression);

        let row = query.fetch_one(&*self.pool).await?;

        let row = row_to_json(&row);
        if row.is_empty() {
            Ok(Value::Null)
        } else {
            row.values()
                .next()
                .ok_or(anyhow::anyhow!("Bad value"))
                .cloned()
        }
    }

    async fn query_row(&self, query: &Query) -> Result<Map<String, Value>> {
        todo!()
    }

    async fn query_col(&self, query: &Query) -> Result<Vec<Value>> {
        todo!()
    }
}
