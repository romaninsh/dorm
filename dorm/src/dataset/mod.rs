//! Datasets are like a Vec<E>, but E are stored remotely and only fetched when needed.
//!
//! When we operate with DataSet we do not know how many rows are in a DataSet.
//! An example dataset could contain all the Orders placed by a client.
//!
//! There are two main traits for Datasets:
//!  - [`ReadableDataSet`]: allows to read rows
//!  - [`WritableDataSet`]: allows to update or delete rows
//!
//! The included implementation for Datasets are:
//!  - [`Table`]: a table is a dataset that stores data in a SQL table and implements both [`ReadableDataSet`] and [`WritableDataSet`].
//!  - [`Query`]: a generic SELECT query that can fetch data and therefore implements [`ReadableDataSet`].
//!
//! [`Table`]: super::table::Table
//! [`Query`]: super::query::Query
mod readable;
pub use readable::ReadableDataSet;

mod writable;
pub use writable::WritableDataSet;
