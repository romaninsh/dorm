use std::sync::Arc;

use crate::model::product::ProductSet;
use dorm::prelude::*;
use tokio_postgres::NoTls;

use anyhow::Result;

mod model;
extern crate dorm;

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

    let sum = expr!("{}::integer + {}::integer", 2, 2);

    let query = Query::new().add_column("sum".to_string(), sum);

    dbg!(&query.preview());
    dbg!(postgres.query_raw(&query).await?);

    println!("Hello from Basic Dorm example!");
    Ok(())
}
