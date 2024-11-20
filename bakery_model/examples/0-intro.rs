use anyhow::Result;
use dorm::{dataset::WritableDataSet, prelude::*};

use bakery_model::*;
use serde::{Deserialize, Serialize};

async fn create_bootstrap_db() -> Result<()> {
    // Run this once for demos to work:
    //  > psql -d postgres -c "CREATE ROLE postgres WITH LOGIN SUPERUSER"
    //
    bakery_model::connect_postgres().await?;
    let dorm_client = bakery_model::postgres();
    let client = dorm_client.client();
    let schema = tokio::fs::read_to_string("bakery_model/schema-pg.sql").await?;
    client.batch_execute(&schema).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    create_bootstrap_db().await?;

    // Welcome to DORM demo.
    //
    // DORM allows you to create types for your "Data Sets". It's easier to explain with example.
    // Your SQL table "clients" contains multiple client records. We do not know if there are
    // 10 clients or 100,000 in the table. We simply refer to them as "set of clients"

    let set_of_clients = Client::table();

    // As you would expect, you can iterate over clients easily.
    for client in set_of_clients.get().await? {
        println!("id: {}, client: {}", client.id, client.name);
    }

    /////////////////////////////////////////////////////////////////////////////////////////
    println!("-------------------------------------------------------------------------------");
    /////////////////////////////////////////////////////////////////////////////////////////

    // In production applications you wouldn't be able to iterate over all the records like this,
    // simply because of the large number of records. Which is why we need to narrow down our
    // set_of_clients:

    let condition = set_of_clients.is_paying_client().eq(&true);
    let paying_clients = set_of_clients.with_condition(condition);

    // Some operation do not require us to fetch all records. For instance if we just need to know
    // count of paying clients we can use count():
    println!(
        "Count of paying clients: {}",
        paying_clients.count().get_one_untyped().await?
    );

    /////////////////////////////////////////////////////////////////////////////////////////
    println!("-------------------------------------------------------------------------------");
    /////////////////////////////////////////////////////////////////////////////////////////

    // Now that you have some idea of what a DataSet is, lets look at how we can reference
    // related sets. Traditionally we could say "one client has many orders". In DORM we say
    // "set of orders that reference set of clients". In this paradigm we only operate with
    // "many-to-many" relationships.

    let orders = paying_clients.ref_orders();

    // Lets pay attention to the type here:
    //  set_of_cilents = Table<Postgres, Client>
    //  paying_clients = Table<Postgres, Client>
    //  orders         = Table<Postgres, Order>
    //
    // Type is automatically inferred, I do not need to specify it. This allows me to define
    // a custom method on Table<Postgres, Order> only and use it like this:

    let report = orders.generate_report().await?;
    println!("Report:\n{}", report);

    // Implementation for `generate_report` method is in bakery_model/src/order.rs and can be
    // used anywhere. Importantly - this file also includes a unit-test for `generate_report`.
    // My test uses a mock data source and is super fast, which is very important for large
    // applications.

    /////////////////////////////////////////////////////////////////////////////////////////
    println!("-------------------------------------------------------------------------------");
    /////////////////////////////////////////////////////////////////////////////////////////

    // One thing that sets DORM apart from other ORMs is that we are super-efficient at building
    // queries. DataSets have a default entity type (in this case - Order) but we can supply
    // our own type:

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    struct MiniOrder {
        id: i64,
        client_id: i64,
    }
    impl Entity for MiniOrder {}

    // Entity (and dependant traits) are needed to load and store "MiniOrder" in our DataSet.
    // Next I'll use `get_some_as` which gets just a single record. The subsequent
    // scary-looking `get_select_query_for_struct` is just to grab and display the query
    // to you, which would look like: SELECT id, client_id FROM .....

    let Some(mini_order) = orders.get_some_as::<MiniOrder>().await? else {
        panic!("No order found");
    };
    println!("data = {:?}", &mini_order);
    println!(
        "MiniOrder query: {}",
        orders
            .get_select_query_for_struct(MiniOrder::default())
            .preview()
    );

    // Next lets assume, that we also want to know "order total" and "client name" in the next
    // use-case.
    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    struct MegaOrder {
        id: i64,
        client_name: String,
        total: i64,
    }
    impl Entity for MegaOrder {}

    let Some(mini_order) = orders.get_some_as::<MegaOrder>().await? else {
        panic!("No order found");
    };
    println!("data = {:?}", &mini_order);
    println!(
        "MegaOrder query: {}",
        orders
            .get_select_query_for_struct(MegaOrder::default())
            .preview()
    );

    // OH WOW!! If you are have managed to run this code:
    //  > cargo run --example 0-intro
    //
    // You might be surprised about thequeries that were generated for you. They look scary!!!!
    //
    // SELECT id, client_id
    // FROM ord
    // WHERE client_id IN (SELECT id FROM client WHERE is_paying_client = true)
    //   AND is_deleted = false;
    //
    // Our struct only needed two fields, so only two fields were queried. That's great.
    //
    // You can also probably understand why "is_paying_client" is set to true. Our Order Set was derived
    // from `paying_clients` Set which was created through adding a condition. Why is `is_deleted` here?
    //
    // As it turns out - our table definition is using extension `SoftDelete`. In the `src/order.rs`:
    //
    //  table.with_extension(SoftDelete::new("is_deleted"));
    //
    // The second query is even more interesting:
    //
    // SELECT id,
    //     (SELECT name FROM client WHERE client.id = ord.client_id) AS client_name,
    //     (SELECT SUM((SELECT price FROM product WHERE id = product_id) * quantity)
    //      FROM order_line WHERE order_line.order_id = ord.id) AS total
    // FROM ord
    // WHERE client_id IN (SELECT id FROM client WHERE is_paying_client = true)
    //   AND is_deleted = false;
    //
    // There is no physical fied for `client_name` and instead DORM sub-queries
    // `client` table to get the name. Why?
    //
    // The implementation is, once again,  inside `src/order.rs` file:
    //
    //  table
    //   .with_one("client", "client_id", || Box::new(Client::table()))
    //   .with_imported_fields("client", &["name"])
    //
    // The final field - `total` is even more interesting - it gathers information from
    // `order_line` that holds quantities and `product` that holds prices.
    //
    // Was there a chunk of SQL hidden somewhere? NO, It's all DORM's query building magic.
    //
    // Look inside `src/order.rs` to see how it is implemented:
    //
    // table
    //   .with_many("line_items", "order_id", || Box::new(LineItem::table()))
    //   .with_expression("total", |t| {
    //     let item = t.sub_line_items();
    //     item.sum(item.total()).render_chunk()
    //   })
    //
    // Something is missing. Where is multiplication? Apparently item.total() is
    // responsible for that, we can see that in `src/lineitem.rs`.
    //
    // table
    //   .with_one("product", "product_id", || Box::new(Product::table()))
    //   .with_expression("total", |t: &Table<Postgres, LineItem>| {
    //      t.price().render_chunk().mul(t.quantity())
    //   })
    //   .with_expression("price", |t| {
    //      let product = t.get_subquery_as::<Product>("product").unwrap();
    //      product.field_query(product.price()).render_chunk()
    //   })
    //
    // We have discovered that behind a developer-friendly and very Rust-intuitive Data Set
    // interface, DORM offers some really powerful features to abstract away complexity.
    //
    // What does that mean to your developer team?
    //
    // You might need one or two developers to craft those entities, but the rest of your
    // team can focus on the business logic - like improving that `generate_report` method!
    //
    // This highlights the purpose of DORM - separation of concerns and abstraction of complexity.
    //
    // Use DORM. No tradeoffs. Productive team! Happy days!

    return Ok(());

    // TODO: Code is slightly more scary here, to be removed

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
