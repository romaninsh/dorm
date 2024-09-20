# Conditions

By default, when you create a `Table` object, it will represent
a set of all rows in the table. Sometimes you want to filter
the rows. Our `product` SQL table contains a `calories` column,
so lets define it and use it for filtering:

```rust
let product = Table::new("product", postgres.clone())
    .with_field("id")
    .with_field("name")
    .with_field("calories");

let low_cal_products = product
    .with_condition(
        product
            .get_field("calories")
            .unwrap()
            .lt(100)
    );

let query = low_cal_products.get_select_query();
writeln!(query.preview());

// renders into: SELECT id, name, calories FROM product WHERE (calories < 100)
```

Condition can be created from various other types, but the most
convenient way is through the use of Operator.

1.  get_field returns a Option<Arc<Field>> object
2.  unwrap() as we know for sure field exists
3.  Arc<Field> implements Operator trait
4.  Operator::lt() returns a Condition object
5.  Condition implements SqlChunk, so it can be part of a query

DORM capabilities allow you to be very flexible with defining
your entities and we are just scratching the surface here.

Before we continue, let me address one annoying bit here. So far
each chapter we have been re-creating our `product` table.

In the real world, you would typically have a pre-defined table
structure, so that no matter how many times you need to operate
with a `product` table, you can easily create it.

DORM has a recommended pattern for generating tables and we
will explore it next.
