use std::sync::OnceLock;

use dorm::prelude::*;

use crate::postgres;

use super::bakery::BakerySet;

pub struct BakerSet {
    pub table: Table<Postgres>,
}
impl BakerSet {
    pub fn new() -> Table<Postgres> {
        BakerSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("bakery", postgres())
                .with_field("name")
                .with_field("contact_details")
                .with_field("bakery_id")
                .has_one_cb("bakery", || {
                    todo!()
                    // BakerySet::new(postgres.clone()).table.with_id(132)
                })
        })
    }

    pub fn name() -> &'static Field {
        BakerSet::table().get_field("name").unwrap()
    }

    pub fn contact_details() -> &'static Field {
        BakerSet::table().get_field("contact_details").unwrap()
    }
}
