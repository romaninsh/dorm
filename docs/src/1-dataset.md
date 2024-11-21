# Data Sets

Most of Rust apps operate with data which is readily available in memory. Business apps
are stateless, loading too much data is an anti-pattern.

Depending of the capabilities of the underlying data storage layer, you should look to
create as many requests and retrieve as little data as possible.

`DataSet` is is a representation of collection of records - stored remotely.

## ReadableDataSet and WritableDataSet

DORM provides two traits: `ReadableDataSet` and `WritableDataSet`. DORM also provides
several implementations of these traits:

- `sql::Table` - implements both `ReadableDataSet` and `WritableDataSet`.
- `sql::Query` - implements `ReadableDataSet` only.

Design of DORM allow you to extend those into NoSQL (like MongoDB or GraphQL sources)
as well as custom RestAPI sources. Those extensions do not need to be part of DORM,
they can be implemented as separate crates.

## Operating with Data sets

At the very basic level - you can iterate through a readable data set.

```rust
let set_of_clients = Client::table();

for client in set_of_clients.get().await? {
    println!("{}", client.name);
}
```

But quite often the advanced persistence layer could allow us to do much more. DORM approach is
to provide ways how one Data Set can yield a different Data Set. Here are some examples:

- Get a set containing subset of records - by filtering.
- Converting `sql::table` into `sql::query` for further manipulation.
- Execute operation over set, such as calculate sum of a field from all records, or create comma-separated list of values.
- Modify or delete multiple records.

DORM prefers to off-load operation execution to the persistence layer, but because this may increase
complexity, DORM also provides a way to abstract this complexity away.

### Example - Lazy expression fields

In our introduction example, we came across an aggregate field: `total`:

```rust
table
  .has_many("line_items", "order_id", || Box::new(LineItem::table()))
  .with_expression("total", |t| {
    let item = t.sub_line_items();
    item.sum(item.total()).render_chunk()
  })
```

Lets examine more generally what is happening here. Use of `has_many` creates

This is a very simple example of a lazy expression field. It is a field that is calculated
by a closure. The closure is passed a reference to the table. The closure can then use
the table to create a new field.

The above example is equivalent to this SQL:

```sql
SELECT id,
    (SELECT SUM((SELECT price FROM product WHERE id = product_id) * quantity)
    FROM order_line WHERE order_line.order_id = ord.id) AS total
FROM ord
```

In this example, we are using a sub-query to calculate the total. The sub-query is
created by calling `sub_line_items()` on the table. This method returns a new table
that is a subset of the original table. The sub-query is then used to create a new
field called `total` that is a sum of the price and quantity.

The implementation
of `sql::Table` however provides ability to create new Data Sets from existing ones.
