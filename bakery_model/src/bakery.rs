use std::sync::{Arc, OnceLock};

use crate::client::ClientSet;
use crate::product::Products;
use dorm::prelude::*;

use crate::postgres;

pub struct BakerySet {}

impl BakerySet {
    pub fn new() -> Table<Postgres> {
        BakerySet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("bakery", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("profit_margin")
                .has_many("clients", "bakery_id", || ClientSet::new())
                .has_many("products", "bakery_id", || Products::new().table())
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

    pub fn ref_clients(&self) -> Table<Postgres> {
        BakerySet::table().get_ref("clients").unwrap()
    }
    pub fn ref_products() -> Products {
        Products::from_table(BakerySet::table().get_ref("products").unwrap())
    }
}
