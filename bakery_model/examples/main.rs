use anyhow::anyhow;
use anyhow::Result;
use dorm::{dataset::WritableDataSet, prelude::*};

use bakery_model::*;

/// This is a helper function to create the database
/// and tables for the bakery model.
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

#[tokio::main]
async fn main() -> Result<()> {
    create_bootstrap_db().await?;

    println!();
    println!("-----------------------------");
    println!("Test1: First we will use a generic table object with \"EmptyEntity\" to manually extract and execute query");
    println!();
    let t = Table::new("bakery", postgres());
    let mut t = t
        .with_id_field("id")
        .with_title_field("name")
        .with_field("profit_margin");
    t.add_condition(t.get_field("profit_margin").unwrap().gt(10));
    let q = t.get_select_query();

    let q = q
        .with_column_field("name")
        .with_condition(expr!("id").eq(&1));
    println!("Q: {}", q.preview());
    println!("R: {}", postgres().query_raw(&q).await.unwrap()[0]);

    println!();
    println!("-----------------------------");
    println!("Test2: Next we will use Bakery::table() with_id(1) to load Bakery {{}} struct. Note that 'id' is not defined in Bakery {{}}");
    println!();

    // Example 1: load a single record from table
    let my_bakery = Bakery::table().with_id(1.into());
    println!(
        "Q: {}",
        my_bakery
            .get_select_query_for_struct(Bakery::default())
            .preview()
    );
    let Some(bakery) = my_bakery.get_some().await? else {
        panic!("No bakery found");
    };

    println!("R: Working for the bakery: {}", bakery.name);
    println!("Note: we will keep using Bakery::table.with_id(1) for referencing further queries");

    println!();
    println!("-----------------------------");
    println!("Test3: We will now traverse into my_bakery.ref_clients() and count how many clients we have");
    println!();
    let clients = my_bakery.ref_clients();
    println!("Q: {}", clients.count().preview());
    println!(
        "R: There are {} clients in this bakery.",
        clients.count().get_one_untyped().await?
    );

    println!();
    println!("-----------------------------");
    println!("Test4: Next we will load: my_bakery.ref_products() supplimenting it .with_inventory() and sum the stock");
    println!();
    // Example 3: referencing products, but augmenting it with a join
    let products = my_bakery.ref_products();
    let products_with_inventory = products.with_inventory();

    println!(
        "Q: {}",
        products_with_inventory
            .sum(products_with_inventory.stock())
            .preview()
    );
    println!(
        "R: There are {} stock in the inventory.",
        products_with_inventory
            .sum(products_with_inventory.stock().clone())
            .get_one_untyped()
            .await?
    );

    // Now for every product, lets calculate how many orders it has
    println!();
    println!("-----------------------------");
    println!("Test5: Next we will double-traverle into my_bakery.ref_clients().ref_orders() and rely on Expression fields to calculate totals");
    println!();

    let clients = my_bakery.ref_clients();
    let orders = clients.ref_orders();

    if false {
        for row in orders.get().await.unwrap().into_iter() {
            println!(
                "id: {}, client: {} (id: {})  total(calculated): {}",
                row.id, row.client_name, row.client_id, row.total
            );
        }
    }

    // DESUGARED:
    let q = orders.get_select_query_for_struct(Order::default());
    println!("Q: {}", q.preview());
    println!();
    println!("R: Orders:");
    println!("-------------------------------------------");
    let res = postgres().query_raw(&q).await?;
    for row_untyped in res.into_iter() {
        let row: Order = serde_json::from_value(row_untyped)?;
        println!(
            "id: {}, client: {:<13} (id: {}) total: {}",
            row.id, row.client_name, row.client_id, row.total
        );
    }

    // now lets delete Doc Brown's orders (using soft delete)
    let client = clients.with_id(2.into());
    let orders = client.ref_orders();

    println!(
        "orders for Doc Brown before delete: {}",
        orders.count().get_one_untyped().await?
    );
    orders.delete().await?;
    println!(
        "orders for Doc Brown after delete: {}",
        orders.count().get_one_untyped().await?
    );

    /*
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
