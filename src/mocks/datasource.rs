use crate::query::Query;
use crate::traits::datasource::DataSource;
use anyhow::Result;
use serde_json::{Map, Value};

pub struct MockDataSource {
    data: Vec<Map<String, Value>>,
}

impl MockDataSource {
    pub fn new(data: &Value) -> MockDataSource {
        let data = data
            .as_array()
            .unwrap()
            .clone()
            .into_iter()
            .map(|x| x.as_object().unwrap().clone())
            .collect();
        MockDataSource { data }
    }

    pub fn data(&self) -> &Vec<Map<String, Value>> {
        &self.data
    }
}

impl<'a> DataSource<'a> for MockDataSource {
    async fn query_fetch(&self, _query: &Query<'a>) -> Result<Vec<Map<String, Value>>> {
        Ok(self.data.clone())
    }

    async fn query_exec(&self, _query: &Query<'a>) -> Result<()> {
        Ok(())
    }

    async fn query_insert(
        &self,
        query: &Query<'a>,
        rows: Vec<Vec<serde_json::Value>>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Query;
    use crate::traits::datasource::DataSource;
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn test_mock_data_source() {
        let json = json!([{
            "name": "John",
            "surname": "Doe"
        },
        {
            "name": "Jane",
            "surname": "Doe"
        }
        ]);

        let data_source = MockDataSource::new(&json);

        let query = Query::new("users")
            .add_column_field("name")
            .add_column_field("surname");
        let result = data_source.query_fetch(&query).await;

        assert_eq!(result.unwrap(), *data_source.data());
    }
}
