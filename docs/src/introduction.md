# Introduction

DORM is an **opinionated business entity framework** for Rust, designed to simplify and
enhance the development of business applications by providing robust, maintainable, and
efficient tools for handling complex business logic and database interactions. It
leverages Rust's type safety and performance to offer a cost-effective and enjoyable
development experience

## Purpose and Opinionated Design

DORM was created with the purpose of transforming how business applications are developed
in Rust. By emphasizing structure, consistency, and best practices, DORM serves as not
just a toolkit but a comprehensive guide for building enterprise-level applications.

As an opinionated business entity framework, DORM prescribes specific methods and patterns
for handling data and business logic. This approach is chosen to ensure that applications
are not only performant and safe but also straightforward to maintain and scale.

Unlike more generic libraries or crates (like Actix or Diesel) DORM focuses on guiding
developers to consistency and best practices in application architecture. DORM provides
an overarching structure that encapsulates more than just individual components, ensuring
that business logic and data management are integrated into a cohesive framework designed
for enterprise applications.

## Architectural Separation of Concern

One of the fundamental principles of DORM is the separation of the data persistence layer
from the business logic. This separation is crucial for several reasons:

- **Flexibility in Data Management** - DORM abstracts the data layer through its robust DataSet
  and Query interfaces, allowing business logic to remain agnostic of the underlying database
  technologies. This abstraction makes it possible to switch underlying databases or adapt to
  different data storage solutions without rewriting business logic.

- **Remote Data Handling** - Acknowledging the trend towards distributed systems, DORM is designed
  to manage data that is often stored remotely and accessed over networks (SQL, NoSQL, GraphQL or
  RestAPI). This design consideration ensures that applications built with DORM can efficiently handle
  data operations across varied environments and scale gracefully as demand increases.

- **Efficiency in Data Operations** - Unlike traditional ORMs, which manage data by frequently
  fetching and storing individual records, DORM optimizes efficiency by maintaining data remotely and
  using complex queries to handle or aggregate data directly in the database. This approach reduces
  the number of database interactions, minimizes data transfer overhead, and enhances overall
  performance by leveraging the database's capabilities to execute operations more effectively.

- **Type Safety and Productivity** - DORM capitalizes on the strengths of Rust’s robust type system,
  enhancing code safety and developer productivity by enforcing type safety across business entities,
  relationships, conditions, and expressions. This integration ensures higher code reliability and
  facilitates faster development through precise type checks.

- **Do not disturb the Business code** - DORM excels in abstracting away the complexities of
  the underlying data structures, ensuring that business logic remains stable and unaffected by
  changes in the database schema. For instance, if the structure of a database is refactored (split
  up table, or endpoint, introduction of cache or switch between database engines)—DORM's
  abstraction layers ensure that these changes do not disrupt the existing business logic. This
  approach not only minimizes disruptions caused by backend modifications but also introduces new
  ways to perform business logic tests through unit-testing.

## Improving the Learning Curve with DORM

DORM solves the challenge of developer learning curve by introducing a structured pattern for
defining business entities using powerful Rust generics. This is a perfect way how your project
structure can appear simple and familiar to developers from OOP backgrounds like Java or C#:

- **Business Entity Object** - Rust has no Objects, but DORM gives a very similar experience
  by leveraging traits and generics. This allows business entities to have the single interface
  to persistence functions (deleting or updating records), typical logic extensions (soft-delete
  and data normalization) and custom developer-defined abstractions (such as order fullfilment)

- **Avoiding borrowing and lifetimes** - Business entities are owned, clonable and can be
  easily shared across your code. They can be further mutated (such as adding more conditions)
  or yield related entities (such as a product having many orders). Rust syntax for manipulating
  entities is simple and easy to understand.

- **Hydrating** - DORM allows you to easily hydrate (or fetch) the data. Business entities are
  defined as sets of remotely stored records. It is easy to iterate, filter or map remote records.
  DORM also allows use of expressions if persistence layer allows subqueries.

## Concepts of DORM

DORM framework relies on concepts that work together and build upon eachother:

1. DataSet - like a Map, but Rows are stored remotely and only fetched when needed.
2. Expressions - recursive template engine for building SQL.
3. Query - a dynamic object representing SQL query.
4. DataSources - an implementation trait for persistence layer.
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

## Simple Example

```rust
use dorm::prelude::*;

let clients = Table::new("client", postgres.clone())
    .add_field("name")
    .add_id_field("id")
    .add_field("active")

let active_clients = clients.add_condition(clients.get_field("active")?.eq(true));

for client in active_clients.get().await? {
    println!("{}", client["name"]?);
}
```

This example relies on concepts of "Table", "Field" to create `clients` DataSet.
In order to target only `active_clients`, we make use of Conditions (which is a type
of Expression) and Field. Finally when fetching data we hydrate into serde_json::Map.

## Same example with Business Entities

Your application is likely to use consistent set of tables and columns. Those can
be defined once and reused through a concept of Business Entities. Lets look how
your code would change with introduction of Business Entity:

```rust
use dorm::prelude::*;
use crate::business_entities::Client;

let clients = Client::table();

let active_clients = clients.only_active();

for client in active_clients.get().await? {
    println!("{}", client.name);
}
```

Defining `clients` now is much simpler. The full set of fields is not needed for our
operation of fetching active clients. We can also define a method `only_active()`
in a business entity crate, so that it would be easy to reuse it across your code.

Finally business entities hydrate into a struct, giving you more type safety.

## Real-life Example

In this book, we will be using a fictional database for your typical Bakery business.
Primarily we will be using `product`, `inventory`, `order` and `client` tables. The
examples will rely on those business entities and focus on demonstrating other
capabilities of DORM:

```rust
fn notify_clients_of_low_stock() -> Result<()> {
    let products = Product::table_with_inventory();
    let products = products.with_condition(products.stock().eq(0));

    let clients = products
        .ref_order()
        .only_active()
        .ref_client();

    for client_comm in clients.get_email_comm().await? {
        client_comm.type = ClientCommType::LowStock;

        client_comm.save_into(ClientComm::queue()).await?;
    }
    Ok(())
}
```

This is more "real-world" example implementing a scalable
implementation for a simple business process of sending emails to
clients that have active orders that cannot be fulfilled due to a low
stock.

The code is simple, safe and maintainable.
