# Fields and Operations

Well, Query stores columns as a Map<Arc<Box<dyn Column>>>
and there are several types that implement `Column` trait,
but the simplest would be a `Field` type:

To add a `Field` to your `Query` you can call `with_column_field`
method:

```rust
let query = Query::new()
    .with_table("product")
    .with_column_field("name")
```

Another type that implements `Column` trait is `Expression`. Lets
modify our query by adding an expression by invoking `with_column` method.
This method is more generic than "with_column_field" and will accept
two arguments - a name of the column and a static `Column` object.

```rust
let person = person
    .with_column("name_caps".to_string(), expr!("UPPER(name)"));
```

If you call `preview()` on your query now, you should see this:

```sql
SELECT name, UPPER(name) AS name_caps FROM product
```

Query is not very long-lived. Once created, it can be executed,
but that's it. More interesting for us - Table objects can
convert themselves into a Query object.

This is our `Table` object from earlier:

```rust

let product = Table::new("product", postgres.clone())
    .with_field("id")
    .with_field("name");
```

You already know that `Table` can produce a `Query`:

```rust
let query = product.get_select_query();
writeln!(query.preview());

// renders into: SELECT id, name FROM product
```

There should be a way to add a "expression" to our query by
defining it in our table, right?

As it turns out, tables are quite lazy when it comes to expressions.
They do not recognize them as fields, and instead will pop them
into your "select_query" just at the last moment and only when
you explicitly need that expresison field. This allows you to
have wide range of expressions in your table and optimize select
query automatically.

```rust
let product = product
    .with_expression(
        "name_caps",
        |t| expr_arc!("UPPER({})", t.get_field("name").unwrap())
    );
```

You have probably noticed, that rather then hard-coding name in
the expression, we are using `get_field` method. This method
will return a &Field object, which we can include in our
ExpressionArc. Field implements `SqlChunk`, right?

Executing `get_select_query()` will will not include `name_caps`
by default. You can explicitly ask which fields you'd like to see
by usign `with_select_query_for_field_names` method:

```rust
let query = product.get_select_query_for_field_names(&["name", "name_caps"]);
writeln!(query.preview());

// renders into: SELECT name, UPPER(name) AS name_caps FROM product
```

## Operations

Relying ot an arbitrary expression macro sometimes is not very
convenient. DORM offers an `Operations` trait, that can be used
to generate some commonly used expressions:

```rust
let product = product
    .with_expression(
        "name_caps",
        |t| t.get_field("name").unwrap().upper()
    );
```

Operations can also be used for generating conditions, lets
look into that next.
