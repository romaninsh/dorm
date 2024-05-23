use std::sync::Arc;

use anyhow::Result;
use dorm::prelude::*;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Serialize, Deserialize, Debug)]
struct Vendor {
    name: String,
    price: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    description: String,
    default_price: String,

    vendors: Vec<Vendor>,
}

struct ProductSet {
    table: Table<Postgres>,
}

impl ProductSet {
    fn new(ds: Postgres) -> Self {
        let table = Table::new("product", ds)
            .with_field("name")
            .with_field("description")
            .with_field("default_price")
            // .has_many_cb("vendors", || {
            //     VendorSet::new().add_condition(VendorSet::product_id.eq(ProductSet::id()))
            // });
            ;

        Self { table }
    }
    pub fn name(&self) -> &Field {
        self.table.get_field("name").unwrap()
    }
    pub fn description(&self) -> &Field {
        self.table.get_field("description").unwrap()
    }
    pub fn default_price(&self) -> &Field {
        self.table.get_field("default_price").unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost dbname=postgres", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let postgres = Postgres::new(Arc::new(Box::new(client)));

    let product_set = ProductSet::new(postgres.clone());

    // let entry = product_set.load_by_id::<Product>(1).await?;

    // entry.description = "blah";
    // entry.save();

    println!("{}", product_set.table.get_select_query().preview());

    Ok(())
}
