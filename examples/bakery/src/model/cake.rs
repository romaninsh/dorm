use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::{model::baker::BakerSet, postgres};

pub struct CakeSet {}
impl CakeSet {
    pub fn new() -> Table<Postgres> {
        CakeSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("cake", postgres())
                .with_field("id")
                .with_field("name")
                .with_field("price")
                .with_field("bakery_id")
                .with_field("gluten_free")
                .with_field("serial")
                .has_many("cake_bakers", "cake_id", || {
                    // add baker_id field into Cake through a left join
                    BakerSet::new().with_join(
                        Table::new("cakes_bakers", postgres())
                            .with_id_field("baker_id")
                            .with_field("cake_id"),
                        "id",
                    )
                    // maybe here some grouping would be more sensible
                })
        })

        // not defining relation to "lineitem" as it has no logical meaning
    }

    pub fn id() -> Arc<Field> {
        CakeSet::table().get_field("id").unwrap()
    }
    pub fn name() -> Arc<Field> {
        CakeSet::table().get_field("name").unwrap()
    }
    pub fn price() -> Arc<Field> {
        CakeSet::table().get_field("price").unwrap()
    }
    pub fn bakery_id() -> Arc<Field> {
        CakeSet::table().get_field("bakery_id").unwrap()
    }
    pub fn gluten_free() -> Arc<Field> {
        CakeSet::table().get_field("gluten_free").unwrap()
    }
    pub fn serial() -> Arc<Field> {
        CakeSet::table().get_field("serial").unwrap()
    }
}
