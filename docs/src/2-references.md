## Table references

DORM allows you to connect tables together:

```rust
let users = ClientSet::new(postgres.clone())
    .with_many("orders", OrderSet::new(postgres.clone()), "user_id", "id")
```

Your "users" table is now connected to orders through a relationship. Remember that since
you are operating with Sets, when you traverse, you get a resulting Set, not a single record.

```rust
let vip_users = users.with_condition(users.field("is_vip").eq(true));
let vip_orders = vip_users.get_ref("orders").unwrap();

let total_order_amount = vip_orders.sum(vip_orders.field("amount")).get_one().await.unwrap();
```

It makes sense to create a bi-directional relationship. Here is how we could define static data sets:

```rust
pub struct ClientSet {}
impl ClientSet {
    pub fn new() -> Table<Postgres> {
        ClientSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("client", postgres())
                .with_id_field("id")
                .with_field("name")
                .with_field("contact_details")
                .with_field("bakery_id")
                .has_many("orders", "client_id", || OrderSet::new())
        })
    }
}
pub struct OrderSet {}
impl OrderSet {
    pub fn new() -> Table<Postgres> {
        OrderSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("ord", postgres())
                .with_field("product_id")
                .with_field("client_id")
                //.has_one("product", "product_id", || Products::new().table())
                .has_one("client", "client_id", || ClientSet::new())
        })
    }
}
```

Since we are opearting with Sets, traversing one-to-many or many-to-one relationship is identical:

```rust
let my_order = OrderSet::new().with_id(123);
let client_data = my_order.get_ref("client").get_row().await.unwrap();
```

On an SQL level, the following happens:

1.  A query is generated for the OrderSet
2.  order query is used as a condition for a client query
3.  client query is executed

```sql
SELECT client_id, name, contact_details FROM client WHERE (order_id IN (SELECT id FROM ord WHERE (product_id = 123)));
```

If your OrderSet contains multiple orders, then "get_ref" set would also contain multiple clients.
This can be useful, for example, lets see how many different clients currently have placed orders:

```rust
let total_clients = OrderSet::new().get_ref("client").count().get_one().await.unwrap();
```

## Subqueries

Knowing relationships between tables can also be useful for creating subqureies. Since those are expensive,
those would be defined as "lazy expressions". Lets look at a simple example:

```rust
let client_set = ClientSet::new()
    .with_expression_before_query("orders_count", move |t| {
        bakery_model::order::OrderSet::new()
            .with_condition(bakery_model::order::OrderSet::client_id().eq(&t.id()))
            .count()
            .render_chunk()
    });

for row in client_set.get_fields(&["name", "orders_count"]).await.unwrap() {
    println!(" name: {}  orders: {}", row["name"], row["orders_count"]);
}
```

You might notice that the call-back code here is using &t.id(), however is not using t.get_ref("client") in any way.

This is because - get_ref here would have returned a set of orders for ALL clients, using "client_id IN (SELECT id FROM client)".
Instead, our condition should be "client_id = client.id()" and to achieve that we should use "get_ref_related("client")". We can
simplify our code:

```rust
let client_set = ClientSet::new()
    .with_expression_before_query("orders_count", move |t| {
        t.get_ref_related("client").count().render_chunk()
    });
```

some more docs here..
