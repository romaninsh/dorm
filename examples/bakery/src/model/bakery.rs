use std::sync::OnceLock;

use dorm::prelude::*;

use crate::{model::cake::CakeSet, postgres};

pub struct BakerySet {}

impl BakerySet {
    pub fn new() -> Table<Postgres> {
        BakerySet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("bakery", postgres())
                .add_field("name")
                .add_field("profit_margin")
                .has_many_cb("cakes", || {
                    CakeSet::new().add_condition(CakeSet::bakery_id().eq(BakerySet::id()))
                })
                .add_field_cb("profit", |t: &Table<Postgres>| {
                    Box::new(t.get_ref("cakes").sum(CakeSet::profit()))
                })
                .has_many_cb("bakers", || BakerySet::new())
        })
    }
    pub fn id() -> &'static Field {
        BakerySet::table().get_field("id").unwrap()
    }
    pub fn name() -> &'static Field {
        BakerySet::table().get_field("name").unwrap()
    }
    pub fn profit_margin() -> &'static Field {
        BakerySet::table().get_field("profit_margin").unwrap()
    }
}
