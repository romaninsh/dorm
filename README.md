# DORM - the Dry Object-Relational Mapper

While most ORM libraries are designed to work specifically with the data by proactively
fetching it, DORM prefers to be lazy and focus on query-mapping and lazy hydration.

## Typical ORM code (Diesel):

Suppose we want to print out list of online users along with a total amount of their basket. Diesel
would require you to fetch lits of users and then iterate over the results potentially producing
many queries:

```rust
let connection = establish_connection();
let thirty_minutes_ago = Utc::now().naive_utc() - chrono::Duration::minutes(30);

let active_users = users
    .filter(last_online.gt(thirty_minutes_ago))
    .load::<User>(&connection)
    .expect("Error loading active users");

for user in active_users {
    let basket_total: f64 = orders
        .filter(user_id.eq(user.id))
        .filter(is_active.eq(false))
        .select(sum(amount))
        .first(&connection)
        .unwrap_or(0.0);

    println!("Active user {} has a basket worth ${:.0}", user.email, basket_total);
}
```

The DORM approach is different, where you can define a query and then iterate over the results. Queries are hydrated only when needed.

```rust
let datasource = dorm::DataSource::new::<Postgres>(establish_connection());
let thirty_minutes_ago = Utc::now().naive_utc() - chrono::Duration::minutes(30);

let orders = datasource.table("user")
    .filter("last_online", ">", thirty_minutes_ago)
    .ref_table("order")
    .filter("is_active", "=", false)

let results = orders
    .query_expr("CONCAT('Active user ', {user.email}, 'has a basket worth ', sum({amount}))")
    .iter_col()

for result in results {
    println!("{}", result);
}
```

DORM also provides syntatic sugar by creating data models for you. Using those
your code would look like this:

```rust
let orders = User::new(datasource)
    .filter(last_online.gt(thirty_minutes_ago))
    .ref_orders()
    .filter(is_active.is_false())
```
