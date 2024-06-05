use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::{
    model::{baker::BakerSet, cake::CakeSet, order::OrderSet},
    postgres,
};

pub struct BakerySet {}

impl BakerySet {
    pub fn new() -> Table<Postgres> {
        BakerySet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("bakery", postgres())
                .with_field("name")
                .with_field("profit_margin")
                .has_many("cakes", "bakery_id", || CakeSet::new())
                .has_many("bakers", "bakery_id", || BakerSet::new())
                .has_many("orders", "bakery_id", || OrderSet::new())
        })
    }
    pub fn id() -> Arc<Field> {
        BakerySet::table().get_field("id").unwrap()
    }
    pub fn name() -> Arc<Field> {
        BakerySet::table().get_field("name").unwrap()
    }
    pub fn profit_margin() -> Arc<Field> {
        BakerySet::table().get_field("profit_margin").unwrap()
    }
}
