# DORM

[![Book](https://github.com/romaninsh/dorm/actions/workflows/book.yaml/badge.svg)](https://romaninsh.github.io/dorm/)

DORM is NOT an ORM!

DORM is a busines entity abstraction framework for Rust. With DORM your
complex business logic that interacts with SQL will be easy-to-read,
decoubled and maintainable.

## What is DORM?

DORM at it's core is a query builder. It dynamically generates and executes
SQL queries - SELECT, UPDATE etc. DORM should work just fine with any schema.
While in development, DORM is using PostgreSQL only.

`Query`, `Expression` and `Operation` are some basic building blocks, but
in most cases you will not need to use them directly.

DORM is also a data source abstraction. By defining a `DataSource`, `Table` and `Field`
you can describe your business entities and their relations. DORM can also
support `Join` for describing complex business entities. In addition
to physical fields, DORM supports SQL and Rust expressions, such as sub-queries
and aggregations.

Think about `Table` not as a physical SQL table, but as a DataSet - collection of records that
your business logic operates on. Because you don't want to repeat your code, you would
want to define your DataSet once and reuse it. We refer to this definition
as an Entity.

A business application will have many entities, which can reference each-other,
and even be composed of one another. This library that you create will be called
an Entity Model.

Once your entities are defined, you can manipulate data through query-building
and controlled execution of those queries for operations like fetching, inserting,
mapping and traversing. To help with data manipulation, you can use a regular
Rust struct to represent a record or associated record.

Using a regular record is great for fetching or inserting data, associated records
is mut and can be saved back into RecordSet.

I suppose at the end DORM implements ORM too.

## DORM use example

In this repository you will find a Bakery example. With that we describe
a Rest API for operating a typical bakery - managing clients, products and
orders. We also provide endpoints for statistics and augment our API with
sufficient data.

Bakery API design is multi-tenant - products, clients and orders are linked
to a specific "Bakery" and API authentication would only allow access to
the data for the specific bakery.

The Bakery example is also used throughot the documentation

`https://romaninsh.github.io/dorm/1-table-and-fields.html`

## How does DORM make things simple? It looks complex!

DORM uses abstraction focusing on a most common use-cases:

```rust
let bakery = Bakery::table().with_id(25);

let clients = bakery.ref_clients();

for client in clients.fetch().await? {
    println!("Client name: {}", client.name);
}
```

For a more complex scenario, where we are provided with product_code and
an amount of new stock, and we must update the inventory. Our table
structure separates inventory.qty from product.code. Traditional ORM
would create some intensive SQL traffic. Alternatively you would need
to write a complex SQL to perform operation manually.

DORM allows you to be both efficient and readable:

```rust
// contains product_code mapped to amount of new inventory
let inventory_shipment: IndexMap<String, i64> = get_shipment();

let bakery = Bakery::table().with_id(25);

let products = bakery.ref_products().with_inventory();

// no need to load all products, just those that are shipped
let shipped_products = products.with_condition(products.code().in_vec(
    inventory_shipment.keys().cloned().collect()
));

shipped_products.map(|product| {
    let qty = inventory_shipment[&product.code];
    product.qty += qty;
}).await?;
```

Here is why DORM is so powerful:

- Code is readable and easy to understand.
- Code is type-safe. It takes advantage of Rusts powerful type system for autocompletion.
- Code is data-safe. It will not cross the boundaries of the Bakery id=25.
- Code is efficient. It will execute just two statements - SELECT and REPLACE, querying
  and modifying only relevant fields and records.
- Code is execution-safe. Both statements are executed as a single transaction.
- Code is implementation-agnostic. Storing "qty" in a separate table (inventory)
  does not make the code look any different.

## DORM features

I continue to describe DORM features in the documentation. Here is a quick
overview:

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

## Current status

DORM currently is in development. See [TODO](TODO.md) for the current status.

## Inspiration

DORM is inspired by Agile Data (from Agile Toolkit):

- `https://www.agiletoolkit.org/data`
- `https://agile-data.readthedocs.io/en/develop/quickstart.html#core-concepts`
