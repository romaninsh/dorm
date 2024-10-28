use crate::postgres;
use dorm::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Product {
    id: i64,
    name: String,
}
impl Entity for Product {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ProductInventory {
    id: i64,
    product_id: i64,
    stock: i64,
}
impl Entity for ProductInventory {}

impl Product {
    pub fn static_table() -> &'static Table<Postgres, Product> {
        static TABLE: OnceLock<Table<Postgres, Product>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new_with_entity("product", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("bakery_id")
            // .has_one("bakery", "bakery_id", || BakerySet::new())
        })
    }
    pub fn table() -> Table<Postgres, Product> {
        Product::static_table().clone()
    }
}

pub trait ProductTable: AnyTable {
    fn with_inventory(self) -> Table<Postgres, ProductInventory>;

    fn id(&self) -> Arc<Field> {
        self.get_field("id").unwrap()
    }
    fn name(&self) -> Arc<Field> {
        self.get_field("name").unwrap()
    }
    fn bakery_id(&self) -> Arc<Field> {
        self.get_field("bakery_id").unwrap()
    }

    // pub fn stock(&self) -> Arc<Field> {
    //     self.get_join("i").unwrap().get_field("stock").unwrap()
    // }
}

impl ProductTable for Table<Postgres, Product> {
    fn with_inventory(self) -> Table<Postgres, ProductInventory> {
        let t = self.into_entity::<ProductInventory>();
        let t = t.with_join(
            Table::new_with_entity("inventory", postgres())
                .with_alias("i")
                .with_id_field("product_id")
                .with_field("stock"),
            "id",
        );
        t
    }
}

pub trait ProductInventoryTable: RelatedTable<Postgres> {
    fn j_stock(&self) -> Table<Postgres, EmptyEntity> {
        let j = self.get_join("i").unwrap();
        j.table().clone()
    }

    fn stock(&self) -> Arc<Field> {
        let j = self.j_stock();
        j.get_field("stock").unwrap()
    }
}
impl ProductInventoryTable for Table<Postgres, ProductInventory> {}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_product() {
//         let table = Product::table();
//         let _field = table.name();

//         let table = table.with_inventory();
//         // let _field = table.stock();
//     }
// }
