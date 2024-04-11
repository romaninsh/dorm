trait DataSource {
    fn execute_query(&self, query: Query) -> Result<DataSourceResult, Error>;
    fn fetch_data(&self, query: DataSourceResult) -> Result<Vec<Vec<String>>, Error>;
}
