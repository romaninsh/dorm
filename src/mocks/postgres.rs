use crate::{Query, Renderable};
use async_trait::async_trait;
use tokio_postgres::Client;

use crate::traits::datasource::DataSource;

#[async_trait]
impl DataSource for Client {
    fn query_fetch(&self, query: &Query) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        let query = query.render();
        let rows = self.query(&query, &[]).await?;

        let mut result = Vec::new();
        for row in rows {
            let mut result_row = Vec::new();
            for col in row.columns() {
                let value: String = row.get(col.name());
                result_row.push(value);
            }
            result.push(result_row);
        }

        Ok(result)
    }

    async fn query_exec(&self, query: &crate::Query) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
