use bakery_model::product::Product;
use dorm::prelude::*;
use serde_json::Value;

#[tokio::main]
async fn main() {
    bakery_model::connect_postgres().await.unwrap();

    let dorm_client = bakery_model::postgres();

    // this is tokio_postgres::Client
    let client = dorm_client.client();

    let schema = tokio::fs::read_to_string("bakery_model/schema-pg.sql")
        .await
        .unwrap();
    client.batch_execute(&schema).await.unwrap();

    // // Ok, now lets work with the models directly
    // let bakery_set = bakery_model::bakery::BakerySet::new();
    // let query = bakery_set.get_select_query();
    // let result = dorm_client.query_raw(&query).await.unwrap();

    // let Some(Value::String(bakery)) = result[0].get("name") else {
    //     panic!("No bakery found");
    // };
    // println!("-----------------------------");
    // println!("Working for the bakery: {}", bakery);
    // println!("");

    // // Now, lets see how many clients bakery has
    // let client_set = bakery_set.get_ref("clients").unwrap();
    // let client_count = client_set.count();

    // println!(
    //     "There are {} clients in the bakery.",
    //     client_count.get_one().await.unwrap()
    // );

    // // Finally lets see how many products we have in the bakery

    // let product_set = bakery_set.get_ref("products").unwrap();
    let product_count = Product::table().count();

    println!(
        "There are {} products in the bakery.",
        product_count.get_one().await.unwrap()
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
}
