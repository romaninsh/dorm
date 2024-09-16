# Table and Fields

In DORM, you define your business entities by creating a Table. A Table is associated with a
physical DataSource and a table. Tables will also contain Fields, Conditions, Joins etc.
Lets start with a simple example:

```rust
use dorm::prelude::*;

let products = Table::new("product", postgres.clone())
    .with_field("name")
    .with_field("description")
    .with_field("default_price");
```

Tables are created by calling `Table::new()` and passing it a table name and a DataSource.

```rust
let products = Table::new("product", postgres.clone())
```

The following invocations to `with_field()` are going to replace your "table" instance
with a new one. Our original code is equivalent to:

```rust
let products = products.with_field("name");
let products = products.with_field("description");
let products = products.with_field("default_price");
```

If you prefer to mutate your table, you can use `add_field()` instead:

```rust
let mut products = Table::new("product", postgres.clone());
products.add_field("name");
products.add_field("description");
products.add_field("default_price");
```

You will see the above method chaining used quite often in DORM. In most cases both methods
will have the identical arguments, but please check with the documentation.

## DataSource

One of the puproses of DORM is to convert your business logic into a set of queries. Because
query languages differ between databases (such as Postgres, MySQL, SQLite, etc), DORM
provides a DataSource abstraction. A DataSource is a trait that defines how to execute
queries and fetch data.

A DataSource for Postgress can be created like this:

```rust
let postgress_client = tokio_postgres::connect("host=localhost dbname=postgres", NoTls)
    .await
    .context("Failed to connect to Postgres")?;

let postgres = Postgres::new(Arc::new(Box::new(postgress_client)));
```

When you create a Table, you pass it a clone of your DataSource. You may use different
DataSources for different tables and DORM will make to execute queries correctly even
if you are traversing relationships between different DataSources.

## What exactly is a Field?

In the example above, we used a pretty generic method to create a field.
In reality, DORM has a very loose definition of what a field is. Lets look at SQL first:

```sql
SELECT id, CONCAT(first_name, ' ', last_name) AS full_name, father.name AS father_name
FROM person
JOIN father ON person.father_id = father.id
```

The above query will return table with 3 columns: id, full_name and father_name. From
the DORM perspective, all three are valid fields.

- id: This is not only a field, but also a "id" field.
- full_name: This field is represented by a SQL expression.
- father_name: This field is imported from another table.

Although the DORM code is a bit complex at this point, I'll still include
it here. Don't worry if you don't understand it all yet, we'll get to
learning about expressions and joins in later chapters.

```rust
let person = Table::new("person", postgres.clone())
    .with_id_field("id")
    .with_field("first_name")
    .with_field("last_name")
    .with_field("father_id");
    .with_expression(
        "full_name",
        expr!("CONCAT({}, ' ', {})",
            &person.get_field("first_name").unwrap(),
            &person.get_field("last_name").unwrap()
        )
    );

let father = person.clone().with_alias("father");

let person_with_father = person
    .with_join("father", father, "father_id")
    .unwrap();

let query = person_with_father.get_select_query_for_field_names(&["id", "full_name", "father_name"]);

writeln!(query.preview());
```

An important takeaway from this example is that we define table along
with the fileds so that we could generate a query out of it. For now,
lets simplify the example and talk about the query generation.

```rust

let person = Table::new("person", postgres.clone())
    .with_field("id")
    .with_field("first_name")
    .with_field("last_name");

let query = person.get_select_query();

writeln!(query.preview());
```

The result of this query should be:

```sql
SELECT id, first_name, last_name FROM person
```

So what is Query?
