use crate::traits::datasource::DataSource;
use crate::Query;
use async_trait::async_trait;
use std::error::Error;

pub struct MockDataSource {
    data: Vec<Vec<String>>,
}

impl MockDataSource {
    pub fn new(data: Vec<Vec<&str>>) -> MockDataSource {
        MockDataSource {
            data: data
                .iter()
                .map(|row| row.iter().map(|s| s.to_string()).collect())
                .collect(),
        }
    }
}

#[async_trait]
impl DataSource for MockDataSource {
    fn query_fetch(&self, _query: &Query) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        Ok(self.data.clone())
    }

    fn query_exec(&self, _query: &Query) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::datasource::DataSource;
    use crate::Query;
    use tokio;

    #[tokio::test]
    async fn test_mock_data_source() {
        let data = vec![vec!["1", "2"]];
        let mut data_source = MockDataSource::new(data.clone());

        let mut query = Query::new("users");
        query.add_column_field("name").add_column_field("surname");
        let result = data_source.query_fetch(&query);

        assert_eq!(result.unwrap(), data);
    }
}
