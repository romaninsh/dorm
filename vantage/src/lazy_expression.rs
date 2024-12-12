use core::fmt;
use std::sync::Arc;

use serde_json::Value;

use crate::{
    prelude::{Expression, Table},
    traits::{datasource::DataSource, entity::Entity},
};

#[derive(Clone)]
pub enum LazyExpression<T: DataSource, E: Entity> {
    AfterQuery(Arc<Box<dyn Fn(&Value) -> Value + Send + Sync + 'static>>),
    BeforeQuery(Arc<Box<dyn Fn(&Table<T, E>) -> Expression + Send + Sync + 'static>>),
}

impl<T: DataSource, E: Entity> fmt::Debug for LazyExpression<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LazyExpression::AfterQuery(_) => f.write_str("AfterQuery(<closure>)"),
            LazyExpression::BeforeQuery(_) => f.write_str("BeforeQuery(<closure>)"),
        }
    }
}
