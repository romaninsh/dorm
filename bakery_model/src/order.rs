use std::sync::{Arc, OnceLock};

use dorm::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    lineitem::{LineItem, LineItemTable},
    postgres, Client, ClientTable,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct Order {
    pub id: i64,
    pub client_id: i64,
    pub client: String,
    pub total: i64,
}
impl Entity for Order {}

impl Order {
    pub fn static_table() -> &'static Table<Postgres, Order> {
        static TABLE: OnceLock<Table<Postgres, Order>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new_with_entity("ord", postgres())
                .with_id_field("id")
                .with_field("client_id")
                .with_extension(SoftDelete::new("is_deleted"))
                .with_expression("total", |t| {
                    let mut item = LineItem::table();
                    item.add_condition(item.order_id().eq(&t.id()));
                    item.get_empty_query()
                        .with_column_arc("total".to_string(), Arc::new(item.total()))
                        .render_chunk()
                })
                .with_expression("client", |t| {
                    let mut client = Client::table();
                    client.add_condition(client.id().eq(&t.client_id()));
                    client.field_query(client.name()).render_chunk()
                })
                .with_one("client", "client_id", || Box::new(Client::table()))
                .with_many("line_items", "order_id", || Box::new(LineItem::table()))
        })
    }
    pub fn table() -> Table<Postgres, Order> {
        Order::static_table().clone()
    }
}

pub trait OrderTable: AnyTable {
    // fn as_table(&self) -> &Table<Postgres, Order> {
    //     self.as_any_ref().downcast_ref().unwrap()
    // }

    fn client_id(&self) -> Arc<Field> {
        Order::table().get_field("client_id").unwrap()
    }
    fn product_id(&self) -> Arc<Field> {
        Order::table().get_field("product_id").unwrap()
    }

    fn ref_client(&self) -> Table<Postgres, Client>;
    fn ref_line_items(&self) -> Table<Postgres, LineItem>;
}

impl OrderTable for Table<Postgres, Order> {
    fn ref_client(&self) -> Table<Postgres, Client> {
        self.get_ref_as("client").unwrap()
    }
    fn ref_line_items(&self) -> Table<Postgres, LineItem> {
        self.get_ref_as("line_items").unwrap()
    }
}
