use std::sync::Arc;

use crate::model::product::ProductSet;
use dorm::prelude::*;
use tokio_postgres::{Error, NoTls};

mod model;
extern crate dorm;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost dbname=postgres", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let postgres = Postgres::new(Arc::new(Box::new(client)));

    let product_set = ProductSet::new(postgres.clone());

    println!("Hello from Basic Dorm example!");
    Ok(())
}
