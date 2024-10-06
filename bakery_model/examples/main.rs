use anyhow::Result;
use bakery_model::*;
use dorm::prelude::*;

/// This is a helper function to create the database
/// and tables for the bakery model.
async fn create_bootstrap_db() -> Result<()> {
    // Connect to postgress and store client statically
    bakery_model::connect_postgres().await?;

    // Get the postgres client for batch execution
    let dorm_client = bakery_model::postgres();
    let client = dorm_client.client();

    // Read the schema from the file and execute it
    let schema = tokio::fs::read_to_string("bakery_model/schema-pg.sql")
        .await
        .unwrap();
    client.batch_execute(&schema).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    create_bootstrap_db().await?;

    // Example 1: load a single record from table
    let my_bakery = Bakery::table().with_id(1.into());
    let Some(bakery) = my_bakery.get_some().await.unwrap() else {
        panic!("No bakery found");
    };

    println!("-----------------------------");
    println!("Working for the bakery: {}", bakery.name);
    println!("");

    // Example 2: referencing other tables from current record set
    let clients = my_bakery.ref_clients();

    println!(
        "There are {} clients in this bakery.",
        clients.count().get_one().await.unwrap()
    );

    // Example 3: referencing products, but augmenting it with a join
    let products = my_bakery.ref_products();
    let products_with_inventory = products.with_inventory();

    println!(
        "There are {} stock in the inventory.",
        products_with_inventory
            .sum(products_with_inventory.stock().clone())
            .get_one()
            .await
            .unwrap()
    );

    /*
    // How many products are there with the name

    // Now for every product, lets calculate how many orders it has

    let product_set = bakery_model::bakery::BakerySet::ref_products();

    println!(
        "Sum of product IDs is {}",
        product_set
            .sum(bakery_model::product::Products::id())
            .get_one()
            .await
            .unwrap()
    );

    // Now lets try to calculate total inventory for all products
    let product_set = bakery_model::bakery::BakerySet::ref_products().with_inventory();
    println!(
        "Total inventory of all products {}",
        product_set
            .sum(product_set.stock())
            .get_one()
            .await
            .unwrap()
    );
    println!();

    println!("-----------------------------");
    println!("Next, lets look at distribution of orders");

    // Next we want to see number of orders for each client
    let mut client_set = bakery_model::client::ClientSet::new().with_alias("a");

    client_set.add_expression("orders_count", move |t| {
        bakery_model::order::OrderSet::new()
            .with_condition(bakery_model::order::OrderSet::client_id().eq(&t.id()))
            .count()
            .render_chunk()
    });

    let q = client_set.get_select_query_for_field_names(&["name", "orders_count"]);
    let res = dorm_client.query_raw(&q).await.unwrap();
    for row in res.into_iter() {
        println!(" name: {}  orders: {}", row["name"], row["orders_count"]);
    }
    */
    Ok(())
}
