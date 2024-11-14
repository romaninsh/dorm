use crate::{order::Order, postgres, Bakery};
use dorm::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Client {
    id: i64,
    name: String,
    contact_details: String,
    bakery_id: i64,
}
impl Entity for Client {}

impl Client {
    pub fn static_table() -> &'static Table<Postgres, Client> {
        static TABLE: OnceLock<Table<Postgres, Client>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new_with_entity("client", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("contact_details")
                .with_field("bakery_id")
                .has_one("bakery", "bakery_id", || Box::new(Bakery::table()))
                .has_many("orders", "client_id", || Box::new(Order::table()))
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
    pub fn table() -> Table<Postgres, Client> {
        Client::static_table().clone()
    }
}

pub trait ClientTable: AnyTable {
    fn as_table(&self) -> &Table<Postgres, Client> {
        self.as_any_ref().downcast_ref().unwrap()
    }
    fn name(&self) -> Arc<Field> {
        self.get_field("name").unwrap()
    }
    fn contact_details(&self) -> Arc<Field> {
        self.get_field("contact_details").unwrap()
    }
    fn bakery_id(&self) -> Arc<Field> {
        self.get_field("bakery_id").unwrap()
    }

    fn ref_bakery(&self) -> Table<Postgres, Bakery> {
        self.as_table().get_ref_as("bakery").unwrap()
    }
    fn ref_orders(&self) -> Table<Postgres, Order> {
        self.as_table().get_ref_as("orders").unwrap()
    }
    // fn ref_cakes(&self) -> Table<Postgres, Cake> {
    //     self.as_table().get_ref_as("cakes").unwrap()
    // }
}
impl ClientTable for Table<Postgres, Client> {}
