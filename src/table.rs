use crate::traits::dataset::WritableDataSet;

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.
pub struct Table {
    table_name: String,

    data_source: Box<dyn DataSource>,
}

impl Table {
    pub fn new(table_name: &str) -> Table {
        Table {
            table_name: table_name.to_string(),
        }
    }

    pub fn
}

impl WritableDataSet for Table {}
