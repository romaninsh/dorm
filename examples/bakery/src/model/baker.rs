use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::{model::cake::CakeSet, postgres};

use super::bakery::BakerySet;

/// Example implementation of a Baker Set for Bakrey model
pub struct BakerSet {}
impl BakerSet {
    pub fn new() -> Table<Postgres> {
        BakerSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("baker", postgres())
                .with_field("name")
                .with_field("contact_details")
                .with_field("bakery_id")
                .has_one("bakery", "bakery_id", || BakerySet::new())
                .has_many("baked_cakes", "baker_id", || {
                    // add baker_id field into Cake through a left join
                    CakeSet::new().with_join(
                        Table::new("cakes_bakers", postgres())
                            .with_id_field("cake_id")
                            .with_field("baker_id"),
                        "id",
                    )
                })
        })
    }

    pub fn name() -> Arc<Field> {
        BakerSet::table().get_field("name").unwrap()
    }

    pub fn contact_details() -> Arc<Field> {
        BakerSet::table().get_field("contact_details").unwrap()
    }
}
