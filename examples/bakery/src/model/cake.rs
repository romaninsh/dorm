use std::sync::OnceLock;

use dorm::prelude::*;

use crate::postgres;

pub struct CakeSet {
    pub table: Table<Postgres>,
}
impl CakeSet {
    pub fn new() -> Table<Postgres> {
        CakeSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("cake", postgres())
                .add_field("id")
                .add_field("name")
                .add_field("profit")
                .add_field("bakery_id")
        })
    }

    pub fn id() -> &'static Field {
        CakeSet::table().get_field("id")
    }

    pub fn name() -> &'static Field {
        CakeSet::table().get_field("name")
    }
    pub fn profit() -> &'static Field {
        CakeSet::table().get_field("profit")
    }
    pub fn bakery_id() -> &'static Field {
        CakeSet::table().get_field("bakery_id")
    }
}
