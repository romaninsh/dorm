use core::fmt;
use std::sync::Arc;

use serde_json::Value;

use crate::{
    prelude::{Expression, Table},
    traits::datasource::DataSource,
};

#[derive(Clone)]
pub enum LazyExpression<T: DataSource> {
    AfterQuery(Arc<Box<dyn Fn(&Value) -> Value + Send + Sync + 'static>>),
    BeforeQuery(Arc<Box<dyn Fn(&Table<T>) -> Expression + Send + Sync + 'static>>),
}

impl<T: DataSource> fmt::Debug for LazyExpression<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LazyExpression::AfterQuery(_) => f.write_str("AfterQuery(<closure>)"),
            LazyExpression::BeforeQuery(_) => f.write_str("BeforeQuery(<closure>)"),
        }
    }
}
