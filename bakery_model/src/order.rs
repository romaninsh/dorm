use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::{
    postgres,
    {bakery::BakerySet, customer::CustomerSet, lineitem::LineitemSet},
};

pub struct OrderSet {}
impl OrderSet {
    pub fn new() -> Table<Postgres> {
        OrderSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("order", postgres())
                .with_field("total")
                .with_field("bakery_id")
                .with_field("customer_id")
                .with_field("placed_at")
                .has_one("customer", "customer_id", || CustomerSet::new())
                .has_one("bakery", "bakery_id", || BakerySet::new())
                .has_many("lineitems", "order_id", || LineitemSet::new())
        })
    }

    pub fn create() -> &'static str {
        "create table if not exists \"order\" (
            id serial primary key,
            total integer not null,
            bakery_id integer not null,
            customer_id integer not null,
            placed_at timestamp not null
        )"
    }

    pub fn customer_id() -> Arc<Field> {
        OrderSet::table().get_field("customer_id").unwrap()
    }

    pub fn bakery_id() -> Arc<Field> {
        OrderSet::table().get_field("bakery_id").unwrap()
    }

    pub fn total() -> Arc<Field> {
        OrderSet::table().get_field("total").unwrap()
    }

    pub fn placed_at() -> Arc<Field> {
        OrderSet::table().get_field("placed_at").unwrap()
    }
}
