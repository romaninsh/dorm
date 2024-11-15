use anyhow::Result;
use dorm::{dataset::WritableDataSet, prelude::*};

use bakery_model::*;

async fn create_bootstrap_db() -> Result<()> {
    // Connect to postgress and store client statically
    bakery_model::connect_postgres().await?;

    // Get the postgres client for batch execution
    let dorm_client = bakery_model::postgres();
    let client = dorm_client.client();

    // Read the schema from the file and execute it
    let schema = tokio::fs::read_to_string("bakery_model/schema-pg.sql").await?;
    client.batch_execute(&schema).await?;

    Ok(())
}

#[tokio::test]
async fn test_abc() {
    panic!("aoeu");
}

#[tokio::main]
async fn main() -> Result<()> {
    create_bootstrap_db().await?;

    panic!("oho");
    println!("In this example, we will be interracting with the records and testing conditions");
    let products = Product::table();

    println!(
        "We are starting with {} products",
        products.count().get_one_untyped().await?
    );

    println!("");
    println!("Adding a single new product");
    let id = products
        .insert(Product {
            name: "Nuclear Sandwich".to_string(),
            calories: 100,
            bakery_id: 1,
            price: 110,
        })
        .await?;

    println!(
        "After adding \"Nuclear Sandwich\" (id={}) we are left with {} products",
        id.unwrap(),
        products.count().get_one_untyped().await?
    );

    Ok(())
}
