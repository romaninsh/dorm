# Data Sets

Traditional ORMs operate with records. If you ever used ORM, you would need to un-learn
that.

DORM operates with Data Sets. A set can contain either a single record, no records or
a huge number of records. As you perform operations on the set, DORM will adjust
the underlying query accordingly.

DORM includes sql::Table and sql::Query - because we all love SQL. However you can
define other data sources - such as NoSQL, REST API, GraphQL, etc. Those extensions
do not need to be in the core DORM library.

For the rest of this book we will only use SQL.

## ReadableDataSet and WritableDataSet

DORM provides two traits: `ReadableDataSet` and `WritableDataSet` and implements
them:

- `sql::Table` - implements both `ReadableDataSet` and `WritableDataSet`.
- `sql::Query` - implements `ReadableDataSet` only.

## Operating with Data sets

At the very basic level - you can iterate through a readable data set.

```rust
let set_of_clients = Client::table();

for client in set_of_clients.get().await? {
    println!("{}", client.name);
}
```

There are more ways to fetch data from `ReadableDataSet`:

- `get` - returns all records in a Vec using default entity type
- `get_as` - return all records using a custom type
- `get_all_untyped` - return all records as a raw JSON object
- `get_some` and `get-some_as` - return only one record (or none)
- `get_row_untyped` - return single record as a raw JSON object
- `get_col_untyped` - return only a single column as a raw JSON values
- `get_one_untyped` - return first column of a first row as a raw JSON value

In most cases you would use use this:

```rust
let client = Client::table().with_id(1);

let client_data = client.get_one().await?;

for client_order in client.orders().get().await? {
    println!("{}", client_order.id);
}
```

## Operating with Queries

Sometimes you would want to tweak a query before executing it.

DORM provides trait TableWithQueries that generates Query objects for you. I will talk about queries
in the later chapters but Query struct is exactly what you think it is - a SQL query.

- `get_empty_query` - returns a query with conditions and joins, but no fields
- `get_select_query` - like `get_empty_query` but adds all physical fields
- `get_select_query_for_field_names` - Provided with a slice of field names and expressions, only includes those into a query.
- `get_select_query_for_field` - Provided a query for individual field or
  expression, which you have to pass through an argument.
- `get_select_query_for_fields` - Provided a query for multiple fields

## Table Fields

I will talk about `Field` in the later chapters, but for now lets say:

- `Field` is a physical field, defined in a table.
- `LazyExpression` is a field defined through a closure returning expression.
- `JoinedTable Field` is a field defined by a joined table.

Unlike Table, Query has no fields.

### Adding physical fields

Field operations are implemented in `TableWithFields` trait.

- `add_field` - adds a field to the table
- `fields` - returns all physical fields
- `add_id_field` and `add_title_field` - adds a field but also labels it
- `id` - return `id` field
- `title` - return `title` field
- `id_with_table_alias` - return `id` field with table alias
- `get_field` - return a field by name
- `get_field_with_table_alias` - return a field by name with table alias
- `search_for_field` - similar to `get_field` but will look for lazy expressions and fields from joined tables.

### Table-only methods and SqlTable

Notably `Table` implements `with_*` methods which are suitable for a builder pattern.
Because those methods return Self, they are not dyn-safe and I haven't included
them into any traits.

- `with_field` - adds a field to the table
- `with_title_field` - adds a title field to the table
- `with_id_field` - adds an id field to the table

```rust
let users = Table::new("users", postgres())
    .with_id_field("id")
    .with_title_field("name")
    .with_field("role_name");
```

Using `with_` is more readable, does not require you to define
`users` as mutable.

There is also a `with` method if you want to define table inside a closure.

```rust
let users = Table::new("users", postgres())
    .with(|t| {
        t.add_field("id");
        t.add_id_field("name");
        t.add_title_field("role_name");
    });
```

`Table` gives you convenience (and some nice type generics), but if you prefer to have a dynamic
object, you can use `SqlTable` trait:

```rust
fn get_some_table() -> Box<dyn SqlTable> {
    if some_condition() {
        Box::new(Table::new("users", postgres()))
    } else {
        Box::new(Table::new("orders", postgres()))
    }
}
```

In the later chapters I'll explain to you how to use this properly.

### Lazy expression fields

I will properly introduce Expressions later. For now think about
expression as a chunk of SQL query:

```sql
let sum = expr!("sum({})", field);
```

Expressions implement `Chunk` trait, which allows them to be used
inside other expressions. Field also implements `Chunk` trait, so you can
pass it into `expr!` macro.

LazyExpression is a mechanism table is using to create on-demand fields.

- `add_expression` - define a callback returning Expression for non-physical field
- `with_expression` - just like `add_expression` but returns modifield Self

Lets define a field that returns current time:

```rust
table.with_expression("current_time", || expr!("now()"));
```

Normally you would rely on `Table`s ability to generate queries or
using Field directly.

In our introduction example, we came across a field: `total`:

```rust
// lineitem.rs
table.with_expression("total", |t: &Table<Postgres, LineItem>| {
    t.price().render_chunk().mul(t.quantity())
})
```

Here `price()` and `quantity()` return unwrapped fields (or something that
implemetns `Column` trait) and Operation::mul() generates an expression
for multiplicating.
