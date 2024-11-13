use crate::{client::*, postgres, product::*};
use dorm::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Bakery {
    pub id: i64,
    pub name: String,
    pub profit_margin: i64,
}
impl Entity for Bakery {}

impl Bakery {
    pub fn static_table() -> &'static Table<Postgres, Bakery> {
        static TABLE: OnceLock<Table<Postgres, Bakery>> = OnceLock::new();

        TABLE.get_or_init(|| {
            let mut t = Table::new_with_entity("bakery", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("profit_margin")
                .has_many("clients", "bakery_id", || Box::new(Client::table()))
                .has_many("products", "bakery_id", || Box::new(Product::table()));
            t.add_condition(t.get_field("profit_margin").unwrap().gt(10));
            t
        })
    }
    pub fn table() -> Table<Postgres, Bakery> {
        Bakery::static_table().clone()
    }
}

pub trait BakeryTable: AnyTable {
    fn as_table(&self) -> &Table<Postgres, Bakery> {
        self.as_any_ref().downcast_ref().unwrap()
    }

    // fields
    fn id(&self) -> Arc<Field> {
        self.get_field("id").unwrap()
    }
    fn name(&self) -> Arc<Field> {
        self.get_field("name").unwrap()
    }
    fn profit_margin(&self) -> Arc<Field> {
        self.get_field("profit_margin").unwrap()
    }

    // references
    fn ref_clients(&self) -> Table<Postgres, Client> {
        self.as_table().get_ref_as("clients").unwrap()
    }
    fn ref_products(&self) -> Table<Postgres, Product> {
        self.as_table().get_ref_as("products").unwrap()
    }
}
impl BakeryTable for Table<Postgres, Bakery> {}
