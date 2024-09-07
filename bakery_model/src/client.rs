use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::postgres;

use super::{bakery::BakerySet, order::OrderSet};

pub struct ClientSet {}
impl ClientSet {
    pub fn new() -> Table<Postgres> {
        ClientSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("client", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("contact_details")
                .with_field("bakery_id")
                .has_one("bakery", "bakery_id", || BakerySet::new())
                .has_many("orders", "client_id", || OrderSet::new())
            // .has_many("baked_cakes", "baker_id", || {
            //     // add baker_id field into Cake through a left join
            //     CakeSet::new().with_join(
            //         Table::new("cakes_bakers", postgres())
            //             .with_id_field("cake_id")
            //             .with_field("baker_id"),
            //         "id",
            //     )
            // })
        })
    }

    pub fn name() -> Arc<Field> {
        ClientSet::table().get_field("name").unwrap()
    }

    pub fn contact_details() -> Arc<Field> {
        ClientSet::table().get_field("contact_details").unwrap()
    }
}
