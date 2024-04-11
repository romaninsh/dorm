use std::error::Error;
use std::result::Result;

use crate::Query;

pub trait DataSource {
    fn query_fetch(&self, query: Query) -> Result<Vec<Vec<String>>, Box<dyn Error>>;
    fn query_exec(&self, query: Query) -> Result<(), Box<dyn Error>>;
}
