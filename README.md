# DORM

[![Book](https://github.com/romaninsh/dorm/actions/workflows/book.yaml/badge.svg)](https://romaninsh.github.io/dorm/)

# Introduction

DORM implements a simple way for your business app developers to access data by using
"Data Set" concept.

It's easier to explain with example. Your SQL table "clients" contains multiple client records. We
do not know if there are 10 clients or 100,000 in the table. We simply refer to them as "set of
clients".

"Set of Clients" is a Rust type:

```rust
let set_of_clients = Client::table();   // Table<Postgres, Client>
```

As you would expect, you can iterate over any set easily:

```rust
for client in set_of_clients.get().await? {   // client: Client
    println!("id: {}, client: {}", client.id, client.name);
}
```

In a production applications you wouldn't be able to iterate over all the records like this,
simply because of the large number of records. Which is why we need to narrow down our
set_of_clients:

```rust
let condition = set_of_clients.is_paying_client().eq(&true);  // condition: Condition
let paying_clients = set_of_clients.with_condition(condition);  // paying_clients: Table<Postgres, Client>
```

We can avoid fetching all records - if we just need to know count of paying clients - we can use count():

```rust
println!(
    "Count of paying clients: {}",
    paying_clients.count().get_one_untyped().await?
);
```

Now that you have some idea of what a DataSet is, lets look at how we can reference
related sets. Traditionally we could say "one client has many orders". In DORM we say
"set of orders that reference set of clients". In this paradigm we only operate with
"many-to-many" relationships.

```rust
let orders = paying_clients.ref_orders();   // orders: Table<Postgres, Order>
```

Type is automatically inferred, I do not need to specify it. This allows me to define
a custom method on Table<Postgres, Order> and use it like this:

```rust
let report = orders.generate_report().await?;
println!("Report:\n{}", report);
```

Implementation for `generate_report` method is in bakery_model/src/order.rs and can be
used anywhere. Importantly - this file also includes a unit-test for `generate_report`.

Tests in DORM mock data source and is super fast. Quick CI/CD process would allow you to
implement more tests.

One thing that sets DORM apart from other ORMs is that we are super-clever at building
queries. DataSets have a default entity type (in this case - Order) but we can supply
our own type:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct MiniOrder {
    id: i64,
    client_id: i64,
}
impl Entity for MiniOrder {}
```

`impl Entity` is needed to load and store "MiniOrder" in any DORM Data Set.
Next I'll use `get_some_as` which gets just a single record. The subsequent
scary-looking `get_select_query_for_struct` is just to grab and display the query
to you:

```rust
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
```

In this next example, I'll only change a few fields of my struct: Remove `client_id` and
add `order_total` and `client_name`:

```rust
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
```

It is now a good time to run this code. Clone this repository and run:

```bash
$ cargo run --example 0-intro
```

You might be surprised about the queries that were generated for you. They look scary!!!!

```sql
SELECT id, client_id
FROM ord
WHERE client_id IN (SELECT id FROM client WHERE is_paying_client = true)
  AND is_deleted = false;
```

Our struct only needed two fields, so only two fields were queried. That's great.

You can also probably understand why "is_paying_client" is set to true. Our Order Set was derived
from `paying_clients` Set which was created through adding a condition. Why is `is_deleted` here?

As it turns out - our table definition is using extension `SoftDelete`. In the `src/order.rs`:

```rust
table.with_extension(SoftDelete::new("is_deleted"));
```

The second query is even more interesting:

```sql
SELECT id,
    (SELECT name FROM client WHERE client.id = ord.client_id) AS client_name,
    (SELECT SUM((SELECT price FROM product WHERE id = product_id) * quantity)
    FROM order_line WHERE order_line.order_id = ord.id) AS total
FROM ord
WHERE client_id IN (SELECT id FROM client WHERE is_paying_client = true)
  AND is_deleted = false;
```

There is no physical fied for `client_name` and instead DORM sub-queries
`client` table to get the name. Why?

The implementation is, once again, inside `src/order.rs` file:

```rust
table
  .with_one("client", "client_id", || Box::new(Client::table()))
  .with_imported_fields("client", &["name"])
```

The final field - `total` is even more interesting - it gathers information from
`order_line` that holds quantities and `product` that holds prices.

Was there a chunk of SQL hidden somewhere? NO, It's all DORM's query building magic.

Look inside `src/order.rs` to see how it is implemented:

```rust
table
  .with_many("line_items", "order_id", || Box::new(LineItem::table()))
  .with_expression("total", |t| {
    let item = t.sub_line_items();
    item.sum(item.total()).render_chunk()
  })
```

Something is missing. Where is multiplication? Apparently item.total() is
responsible for that, we can see that in `src/lineitem.rs`.

```rust
table
  .with_one("product", "product_id", || Box::new(Product::table()))
  .with_expression("total", |t: &Table<Postgres, LineItem>| {
    t.price().render_chunk().mul(t.quantity())
  })
  .with_expression("price", |t| {
    let product = t.get_subquery_as::<Product>("product").unwrap();
    product.field_query(product.price()).render_chunk()
  })
```

We have discovered that behind a developer-friendly and very Rust-intuitive Data Set
interface, DORM offers some really powerful features to abstract away complexity.

What does that mean to your developer team?

You might need one or two developers to craft those entities, but the rest of your
team can focus on the business logic - like improving that `generate_report` method!

This highlights the purpose of DORM - separation of concerns and abstraction of complexity.

Use DORM. No tradeoffs. Productive team! Happy days!

## Concepts of DORM

To understand DORM in-depth, you would need to dissect and dig into its individual components:

1. DataSet - like a Map, but Rows are stored remotely and only fetched when needed.
2. Expressions - recursive template engine for building SQL.
3. Query - a dynamic object representing a single SQL query.
4. DataSources - an implementation trait for persistence layer. Can be Postgres, a mock (more implementations coming soon).
5. Table - DataSet with consistent columns, condition, joins and other features of SQL table.
6. Field - representing columns or arbitrary expressions in a Table.
7. Busines Entity - a record for a specific DataSet (or Table), such as Product, Order or Client.
8. CRUD operations - insert, update and delete records in DataSet through hydration.
9. Reference - ability for DataSet to return related DataSet (get client emails with active orders for unavailable stock items)
10. Joins - combining two Tables into a single Table without hydration.
11. Associated expression - Expression for specific DataSource created by operation on DataSet (sum of all unpaid invoices)
12. Subqueries - Field for a Table represented through Associated expression on a Referenced DataSet.
13. Aggregation - Creating new table from subqueries over some other DataSet.
14. Associated record - Business Entity for a specific DataSet, that can be modified and saved back.

Depending on your use pattern, you would be using several of the above concepts. The rest of this
book will focus on one concept at a time and will discuss it in depth.

The base use pattern of DORM, however would be primarily around Business Entities, Tables and Fields only.

## Using DORM in your app

To start a new app using DORM, you can use this code:

```rust
use dorm::prelude::*;

let postgres = Postgres::new(Arc::new(Box::new(tokio_postgres_client)));

let mut clients = Table::new("client", postgres.clone())
    .with_field("name")
    .with_id_field("id")
    .with_field("active")

let active_clients = clients.add_condition(clients.get_field("active")?.eq(&true));

for client in active_clients.get_all_untyped().await? {
    println!("{}", client["name"]?);
}
```

Typically you would want to abstract away initialization of `Table`, like we
did in the `bakery_example`. Once you do that, your code should look like this:

```rust
let clients = Client::table(); // clients: Table<Postgres, Client>
let active_clients = clients.only_active();

for client in active_clients.get().await? {
    println!("{}", client.name);
}
```

# Learning DORM

I have wrote a detailed Book for DORM, to introduce each concept in great detail:

1. [DataSet abstraction](https://romaninsh.github.io/dorm/1-table-and-fields.html) - like a Map, but Rows are stored remotely and only fetched when needed.
2. [Expressions](https://romaninsh.github.io/dorm/2-expressions-and-queries.html) - use a power of SQL without writing SQL.
3. [Query](https://romaninsh.github.io/dorm/2-expressions-and-queries.html#how-query-uses-expressions-) - a structured query-language aware object for any SQL statement.
4. [DataSources](https://romaninsh.github.io/dorm/1-table-and-fields.html#datasource) - a way to abstract away the database implementation.
5. [Table](https://romaninsh.github.io/dorm/1-table-and-fields.html) - your in-rust version of SQL table or a view
6. [Field](https://romaninsh.github.io/dorm/3-fields-and-operations.html) - representing columns or arbitrary expressions in a data set.
7. WIP: Entity modeling - a pattern for you to create your onw business entities.
8. TODO: CRUD operations - serde-compatible insert, update and delete operations.
9. [Joins](https://romaninsh.github.io/dorm/6-joins.html) - combining tables into a single table.
10. WIP: Reference traversal - convert a set of records into a set of related records.
11. WIP: Subqueries - augment a table with expressions based on related tables.

Additionally, DORM has a full documentation. Just run: `cargo doc --open` and you will see
all the details.

## Current status

DORM currently is in development. See [TODO](TODO.md) for the current status.

## Author

DORM is implemented by **Romans Malinovskis**. To get in touch:

- <https://www.linkedin.com/in/romansmalinovskis>
- <https://bsky.app/profile/nearly.guru>
