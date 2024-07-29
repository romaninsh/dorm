use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::postgres;

pub struct CakesBakersSet {}
impl CakesBakersSet {
    pub fn new() -> Table<Postgres> {
        CakesBakersSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("cakes_bakers", postgres())
                .with_id_field("cake_id")
                .with_field("baker_id")
        })
    }

    pub fn create() -> &'static str {
        "create table if not exists cakes_bakers (
            cake_id integer not null,
            baker_id integer not null
        )"
    }

    pub fn cake_id() -> Arc<Field> {
        CakesBakersSet::table().get_field("cake_id").unwrap()
    }

    pub fn baker_id() -> Arc<Field> {
        CakesBakersSet::table().get_field("baker_id").unwrap()
    }
}
