use crate::Query;

// Represents a dataset that may generate query to fetch data
pub trait ReadableDataSet: IntoIterator<Item = Vec<String>> {
    fn select_query(&self) -> Query;
}

// Represents a dataset that may also be modified through a query
pub trait WritableDataSet: ReadableDataSet {
    fn insert_query(&self) -> Query;
    fn update_query(&self) -> Query;
    fn delete_query(&self) -> Query;
}
