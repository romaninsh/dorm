use crate::{postgres, product::Product};
use dorm::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Bakery {
    id: i64,
    name: String,
    profit_margin: String,
}
impl Entity for Bakery {}

impl Bakery {
    pub fn static_table() -> &'static Table<Postgres, Bakery> {
        static TABLE: OnceLock<Table<Postgres, Bakery>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new_with_entity("bakery", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("profit_margin")
                // .has_many("clients", "bakery_id", || ClientSet::new())
                .has_many("products", "bakery_id", || Box::new(Product::table()))
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
    fn id(&self) -> &Arc<Field> {
        self.get_field("id").unwrap()
    }
    fn name(&self) -> &Arc<Field> {
        self.get_field("name").unwrap()
    }
    fn profit_margin(&self) -> &Arc<Field> {
        self.get_field("profit_margin").unwrap()
    }

    // fn ref_clients(&self) -> Table<Postgres> {
    //     self.table().get_ref("clients").unwrap()
    // }
    fn ref_products(&self) -> Table<Postgres, Product> {
        self.as_table().get_ref_as("products").unwrap()
    }
}
impl BakeryTable for Table<Postgres, Bakery> {}

fn test() {
    let table = Bakery::table();
    let products = table.ref_products();
}
