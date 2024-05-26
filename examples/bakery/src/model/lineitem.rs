use std::sync::OnceLock;

use dorm::prelude::*;

use crate::{
    model::{cake::CakeSet, order::OrderSet},
    postgres,
};

pub struct LineitemSet {}
impl LineitemSet {
    pub fn new() -> Table<Postgres> {
        LineitemSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("lineitem", postgres())
                .with_field("price")
                .with_field("quantity")
                .with_field("order_id")
                .with_field("cake_id")
                .has_one("order", "order_id", || OrderSet::new())
                .has_one("cake", "cake_id", || CakeSet::new())
        })
    }

    pub fn order_id() -> &'static Field {
        LineitemSet::table().get_field("order_id").unwrap()
    }

    pub fn cake_id() -> &'static Field {
        LineitemSet::table().get_field("cake_id").unwrap()
    }

    pub fn quantity() -> &'static Field {
        LineitemSet::table().get_field("quantity").unwrap()
    }

    pub fn price() -> &'static Field {
        LineitemSet::table().get_field("price").unwrap()
    }
}
