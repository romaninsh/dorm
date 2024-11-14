use std::sync::{Arc, OnceLock};

use dorm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{order::Order, postgres, Product, ProductTable};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct LineItem {
    pub id: i64,
    pub price: i64,
    pub quantity: i64,
    pub order_id: i64,
}

impl Entity for LineItem {}

impl LineItem {
    pub fn static_table() -> &'static Table<Postgres, LineItem> {
        static TABLE: OnceLock<Table<Postgres, LineItem>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new_with_entity("order_line", postgres())
                .with_field("quantity")
                .with_field("order_id")
                .with_field("product_id")
                .with_expression("total", |t: &Table<Postgres, LineItem>| {
                    t.price().render_chunk().mul(t.quantity())
                })
                .with_expression("price", |t| {
                    let mut product = Product::table();
                    product.add_condition(product.id().eq(&t.product_id()));
                    product.field_query(product.price()).render_chunk()
                })
                .has_one("order", "order_id", || Box::new(Order::table()))
                .has_one("product", "product_id", || Box::new(Product::table()))
        })
    }
    pub fn table() -> Table<Postgres, LineItem> {
        LineItem::static_table().clone()
    }
}

pub trait LineItemTable: AnyTable {
    fn as_table(&self) -> &Table<Postgres, LineItem> {
        self.as_any_ref().downcast_ref().unwrap()
    }
    fn quantity(&self) -> Arc<Field> {
        self.get_field("quantity").unwrap()
    }
    fn order_id(&self) -> Arc<Field> {
        self.get_field("order_id").unwrap()
    }
    fn product_id(&self) -> Arc<Field> {
        self.get_field("product_id").unwrap()
    }
    fn total(&self) -> Box<dyn Column> {
        self.as_table().search_for_field("total").unwrap()
    }
    fn price(&self) -> Box<dyn Column>;
}
impl LineItemTable for Table<Postgres, LineItem> {
    fn price(&self) -> Box<dyn Column> {
        self.search_for_field("price").unwrap()
    }
}
