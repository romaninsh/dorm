use std::sync::Arc;

use crate::model::product::ProductSet;
use dorm::prelude::*;
use rust_decimal::Decimal;
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

    let product_with_certain_price = product_set.add_condition(
        product_set
            .price()
            .gt(Decimal::new(10, 0))
            .and(product_set.price().lt(Decimal::new(100, 0))),
    );

    let product_price_sum = product_set.sum(product_set.price()).get_one().await?;
    let special_price_sum = product_with_certain_price
        .sum(product_set.price())
        .get_one()
        .await?;

    println!("Sum of prices: {product_price_sum}, sum of special prices: {special_price_sum}");
    Ok(())
}
