# Queries

In DORM, query is a dynamic representation of a SQL query. You already
saw how to create a query in the previous chapter, but now we will
learn how to create query from scratch.

## Expressions

Expression is a building block of a query as well as a template engine
for your query parameters. Lets start with a simple example:

```rust
let expression = Expression::new(
    "SELECT * FROM users WHERE name = {}",
    vec![json!("John")]);

writeln!(expression.preview());
```

The above expression will be rendered as:

```sql
SELECT * FROM users WHERE name = 'John'
```

Expressions do not know anything about the underlying database and
they cannot execute themselves. Parameters you are passing, must be
of type `serde_json::Value`.

To simplify the process DORM offers you a `expr!` macro:

```rust
let expression = expr!("SELECT * FROM users WHERE name = {}", "John");
```

The parameters to `expr!` macro can be any owned scalar types, as long
as they can be converted to `serde_json::Value` using `serde_json::json!`.
macro.

While convenient, there is a significant limitation to Expressions -
they cannot be nested. This is because Expression cannot render itself
into a json::Value.

To overcome this limitation, DORM offers a ExpressionArc type.

## Expression Arc

As the name implies, ExpressionAarc keeps its parameters inside an Arc
and therefore parameters can be dynamic objects. Anything that implements
`SqlChunk` trait can be used as a parameter.

Naturally both `Expression` and `ExpressionArc` implement `SqlChunk`, but
there are more types that implement `SqlChunk` trait and we will look
at them later.

ExpressionArc can be created through a `expr_arc!` macro:

```rust
let expression = expr_arc!("SELECT * FROM users WHERE name = {}", "John");
writeln!(expression.preview());

// renders into: SELECT * FROM users WHERE name = 'John'
```

You can now pass expresisons recursively:

```rust
let condition = expr_arc!("name = {}", "John");
let expression = expr_arc!("SELECT * FROM users WHERE {}", condition);
writeln!(expression.preview());

// renders into: SELECT * FROM users WHERE name = 'John'
```

You might have noticed, that nested expressions are not escaped.
Both Expression and ExpressionArc can cloned and passed around freely.

## Flattening Expressions

As you can see in the example above, expression can be nested. When
we need to send off expression to the database, we need to flattern it.

For that, DORM offers a `render_chunk()` method on `SqlChunk` trait.
This method will convert a dynamic `SqlChunk` into a static `Expression`
type:

```rust
let condition = expr_arc!("name = {}", "John");
let expression = expr_arc!("SELECT * FROM users WHERE {}", condition);
let flattened = expression.render_chunk();

dbg!(flattened.sql());
dbg!(flattened.params());

// renders into: SELECT * FROM users WHERE name = {}
// params: [json!("John")]
```

In the example above, we used `render_chunk()` method on `ExpressionArc`
to convert it into a static `Expression` type. Then sql() and params()
methods were called to get the final template and parameters. Template
has correctly combined nested condition, while leaving parameter value
separated.

## How Query uses Expressions ?

A query object is designed as a template engine. It contains maps
of various columns, conditions, joins etc. Query implements `SqlChunk`
and query itself can be contained inside expression or another query:

```rust
let expr1 = expr!("name = {}", "John");
let expr2 = expr!("age > {}", 30);

let query = Query::new()
    .with_table("users", None)
    .with_column_field("id")
    .with_column_field("name")
    .with_condition(expr1)
    .with_condition(expr2);

writeln!(query.preview());

// renders into: SELECT id, name FROM users WHERE name = 'John' AND age > 30
```

Query does not know anything about the underlying database and therefore
cannot execute itself. It can only be rendered into a template and
parameters.

Query is immutable calling `with_*` methods will take the ownership,
modify and return a new instance, making it perfect for chaining.

Methods like `with_condition` can accept any argument
that implements `SqlChunk` trait, lets create another query,
based on the one we had above:

```rust
// query is from example above
let query2 = Query::new()
    .with_table("orders", None)
    .with_condition(expr_arc!("user_id in {}",
        query
            .clone()
            .without_columns()
            .with_column_field("id")
    ));

writeln!(query2.preview());

// renders into: SELECT * FROM orders WHERE user_id in (SELECT id, name, age FROM users WHERE name = 'John' AND age > 30)
```

Importantly - the two parameters which were set (and then cloned)
for the `query` are kept separate from a final query rendering and
will be passed into DataSource separately. This ensures that
SQL injection is never possible.
