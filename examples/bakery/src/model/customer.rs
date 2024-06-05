use std::sync::{Arc, OnceLock};

use dorm::prelude::*;

use crate::{model::order::OrderSet, postgres};

pub struct CustomerSet {}
impl CustomerSet {
    pub fn new() -> Table<Postgres> {
        CustomerSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("customer", postgres())
                .with_field("name")
                .with_field("notes")
                .has_many("orders", "customer_id", || OrderSet::new())
        })
    }

    pub fn name() -> Arc<Field> {
        CustomerSet::table().get_field("name").unwrap()
    }

    pub fn contact_details() -> Arc<Field> {
        CustomerSet::table().get_field("contact_details").unwrap()
    }
}
