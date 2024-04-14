use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.
pub struct Table {
    data_source: Box<dyn DataSource>,
    table_name: String,
    fields: Vec<String>,
}

impl Table {
    pub fn new(table_name: &str, data_source: Box<dyn DataSource>) -> Table {
        Table {
            table_name: table_name.to_string(),
            data_source,
            fields: Vec::new(),
        }
    }

    pub fn add_field(mut self, field: &str) -> Self {
        self.fields.push(field.to_string());
        self
    }

    pub fn get_select_query(&self) -> crate::Query {
        let mut query = crate::Query::new(&self.table_name);
        for field in &self.fields {
            query.add_column_field(field);
        }
        query
    }

    pub fn get_all_data(&self) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        self.data_source.query_fetch(&self.get_select_query())
    }

    pub fn iter(&self) -> impl Iterator<Item = Vec<String>> {
        self.get_all_data().unwrap().into_iter()
    }
}

impl IntoIterator for Table {
    type Item = Vec<String>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.get_all_data().unwrap().into_iter()
    }
}

impl ReadableDataSet for Table {
    fn select_query(&self) -> crate::Query {
        self.get_select_query()
    }
}

impl WritableDataSet for Table {
    fn insert_query(&self) -> crate::Query {
        todo!()
    }

    fn update_query(&self) -> crate::Query {
        todo!()
    }

    fn delete_query(&self) -> crate::Query {
        todo!()
    }
}
