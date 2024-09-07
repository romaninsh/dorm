use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::{
    postgres,
    {client::ClientSet, product::Products},
};

pub struct OrderSet {}
impl OrderSet {
    pub fn new() -> Table<Postgres> {
        OrderSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("ord", postgres())
                .with_field("product_id")
                .with_field("client_id")
                .has_one("product", "product_id", || Products::new().table())
                .has_one("client", "client_id", || ClientSet::new())
            // .has_many("lineitems", "order_id", || LineitemSet::new())
        })
    }

    pub fn client_id() -> Arc<Field> {
        OrderSet::table().get_field("client_id").unwrap()
    }
}
