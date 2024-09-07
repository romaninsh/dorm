use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

use crate::bakery::BakerySet;
use dorm::prelude::*;

use crate::postgres;

#[derive(Debug)]
pub struct Products {
    table: Table<Postgres>,
}
impl Products {
    pub fn new() -> Products {
        Products {
            table: Products::static_table().clone(),
        }
    }
    pub fn from_table(table: Table<Postgres>) -> Self {
        Self { table }
    }
    pub fn static_table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("product", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("bakery_id")
                .has_one("bakery", "bakery_id", || BakerySet::new())
        })
    }
    pub fn table(&self) -> Table<Postgres> {
        self.table.clone()
    }
    pub fn mod_table(self, func: impl FnOnce(Table<Postgres>) -> Table<Postgres>) -> Self {
        let table = self.table.clone();
        let table = func(table);
        Self { table }
    }
    pub fn with_inventory(self) -> Self {
        self.mod_table(|t| {
            t.with_join(
                Table::new("inventory", postgres())
                    .with_alias("i")
                    .with_id_field("product_id")
                    .with_field("stock"),
                "id",
            )
        })
    }

    pub fn id() -> Arc<Field> {
        Products::static_table().get_field("id").unwrap()
    }
    pub fn name() -> Arc<Field> {
        Products::static_table().get_field("name").unwrap()
    }
    pub fn bakery_id() -> Arc<Field> {
        Products::static_table().get_field("bakery_id").unwrap()
    }

    pub fn stock(&self) -> Arc<Field> {
        self.get_join("i").unwrap().get_field("stock").unwrap()
    }
}

impl Deref for Products {
    type Target = Table<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}
