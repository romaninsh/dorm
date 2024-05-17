# Introduction

DORM brings a powerful business domain abstraction layer to Rust.

For a business application, it is important to have flexibility, stability of the code-base,
ability to easily maintain and change the code, remain decoupled from implementation detail
and most importantly to have a consise and easy to operate with syntax.

DORM is loosely based on Agile Toolkit (https://agiletoolkit.org/)

## The Query Languages

Traditionally ORM libraries simplify interaction with the database, reducing your powerful
SQL database to a simple key-value store. DORM takes a different approach. It takes advantage
of the SQL language to convert your Rust code into powerful SQL queries reducing number of
queries your application would need to execute.

DORM contains 3 layers of abstraction:

 1. Expressions - a parametric template system with recursive rendering capabilities.
 2. Query - a structured query-language aware object, that can be manipulated into any query.
 3. DataSets and Models - native Rust structures for interactign with single or multiple records.

 Operations ond DataSets and Models are translated into SQL queries, which are then converted into
 expressions.

 To understand the basics of DORM, lets start with the fundamentals of expressions.

 ## Expressions

 There are two base classes that Expressions are built around:

  - Expression - a full ownership expression, which parameters have type of Value.
  - ExpressionArc - a shared ownership expression, which parameters can be converted into Expression

Expressions are short-lived - they are created, rendered and discarded. ExpressionArc can remain
in memory for longer time and tie together various conditions, that may have shared ownership and
can be modified from other parts of your application, such as quick-search field.

Lets create an expression first:

```rust
use dorm::prelude::*;

let expr = expr!("concat({}, {})", "hello", "world");

println!("{}", expr.preview());
```

When expression is created, the template is stored separately from the arguments. This allows
you to use arbitrary types as arguments and also use nested queries too:

```rust
let expr = expr!(
    "concat({}, {})",
    expr!("upper({})", "hello"),
    "world");

let expr2 = expr!("{} + {}", 2, 3);
```

Method `preview()` would insert the arguments into the template, but when actually executing the query, the inserting would be done by the database driver instead.

JSON values are also natively supported. You may also implement `SqlChunk` trait for your own types
to allow them to be used in expressions too.

```rust
let expr = expr!("json = {}", json!({"name": "John", "age": 25}));
```

For instance, you can use Operation for creating nested expressions:

```rust
// (name = 'John' AND age > 25)
let expr_and = Operation::and(vec![
    expr!("name = {}", "John"),
    expr!("age > {}", 25)
]);

// concat('hello', 'world')
let fx_call = Operation::fx("concat", vec![
    expr!("hello"), expr!("world")
]);
```

Now that you understand that expression is a building block, lets move on to Query.

## Query

Query is a way to dynamically build query:

```rust
let query = Query::new()
    .set_table("users")
    .add_column_field("id")
    .add_column_field("email")
    .add_column("full_name", Some(expr!("concat(first_name, ' ', last_name)")))
    .add_where(expr!("age > {}", 18))
    .add_order_by(expr!("full_name"))
```

Note: Query (and several other structures) use standard rust builder pattern. Methods such
as add_column_field do not modify the object, but consume old object and retur new one.

Since Query is built with Expressions, which are recrursive by design, Queries can often be nested:

```rust
let roles = Query::new()
    .set_table("roles")
    .add_column_field("id")
    .add_column_field("role_name");

let outer_query = Query::new()
    .set_table("users")
    .add_with("roles", roles)
    .add_join(JoinQuery::new(
        JoinType::Inner,
        QuerySource::Table("roles".to_string(), None),
        QueryConditions::on().add_condition(expr!("users.role_id = roles.id")),
    ))
    .add_column_field("user_name")
    .add_column_field("roles.role_name");

let group = Query::new()
    .set_source(QuerySource::Query(outer_query))
    .add_column_field("role_name")
    .add_column_field("c", Operation::count("*"))
    .add_group_by("role_name");
```

Query cannot execute itself, but a friendly DataSource can execute your query for you:

```rust
// client is a tokio-postgres client
let postgres = Postgres::new(Arc::new(Box::new(client)));

let result = postgres.select_rows(&group).await.unwrap();
```

You also have some flexibility here - Query (assumes SQL query) can be executed by Postgres DataSource or MySQL DataSource. However - there may also be GQuery (assumes GQL) that could
have unique implementaiton and it would need an appropriate DataSource to be executed.

Result will contain Vec<Value>, where Value will be a Value::Hashmap with keys being column names,
however for a different databases the Value could be different.

## DataSets and Models

The final layer of DORM is DataSets (and Models). DataSet represents a collection of records, while
a Model represents a single record.

Model is merely a simple rust Struct that implements Serialise and Deserialise traits, however
before you can use a Model you will need to declare a DataSet.

A simplest DataSet implementation is offered by Table:

```rust
let products = Table::new("product", postgres.clone())
    .add_field("name")
    .add_field("description")
    .add_field("default_price")
```

Note: current version of DORM does not support field types, however this is something that will
be added in later.

You can create clone of a table and tweak things up:

```rust
let expensive_products = products.clone()
    .add_condition(products.get_field("default_price")?.gt(100));
```

Note: `products.get_field()` returns Option<Field> and can be used to build expressions. Because
the field you specify may not exist, you need to use `?` or `unwrap()`. Typically this is hidden
behind explicit methods of a custom struct.

Declaring Set Struct that corresponds to your business models is a great way to keep your code
clean and embedd some business logic:

```rust
pub struct ProductSet {
    table: Table<Postgres>,
}
impl ProductSet {
    pub fn new(ds: Postgres) -> Self {
        let table = Table::new("product", ds)
            .add_field("name")
            .add_field("description")
            .add_field("default_price");
        Self { table }
    }

    pub fn name(&self) -> &Field {
        self.get_field("name").unwrap()
    }

    pub fn description(&self) -> &Field {
        self.get_field("description").unwrap()
    }

    pub fn price(&self) -> &Field {
        self.get_field("default_price").unwrap()
    }
}

pub struct ExpensiveProductSet {
    table: Table<Postgres>,
    price_threshold: Value::Integer,
}
impl ExpensiveProductSet {
    pub fn new(ds: Postgres, price_threshold: Value::Integer) -> Self {
        let table = ProductSet::new(ds);
        let table = table
            .add_condition(table.default_price().gt(price_threshold));
        Self { table, price_threshold }
    }

    pub fn name(&self) -> &Field {
        self.get_field("name").unwrap()
    }

    pub fn description(&self) -> &Field {
        self.get_field("description").unwrap()
    }

    pub fn price(&self) -> &Field {
        self.get_field("default_price").unwrap()
    }
}
```

Note: at some point there should be a macro for unrolling field methods. Now that you
have defined your sets, you can easily work with them:

```rust
let product_data = ProductSet::new(postgres.clone()).get().await.unwrap(); // Vec<Value>

let expensive_sum = ExpensiveProductSet::new(postgres.clone(),100)
    .sum("default_price")
    .await
    .unwrap(); // Value::Integer
```

Finally, lets see how you can work with Models:

```rust
#[derive(Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub description: String,
    pub default_price: i32,
}

// Increase our expensive prices a little
ExpensiveProductSet::new(postgres.clone(),100)
    .map::<Product>(|p|p.default_price += 10)
    .await
    .unwrap();
```

## Relations and Nested Models

The rest of DSQL is designed to help you map your business logic into models. For instance, you
may want to join several tables.

```rust
let basket_items = Table::new("basket_item", postgres.clone())
    .add_field("basket_id")
    .add_field("product_id")
    .add_field("quantity");

let basket_items = basket_items.add_join(ProductSet::new(postgres.clone()), "product_id", "id");
```

You can also declare a relation between tables, rather then joining them:

```rust
let basket_items = basket_items.has_one(
    "basket",
    Table::new("basket", postgres.clone()).add_field("date"),
    "basket_id",
    "id");
```

Relationship does not have any impact on the generated query, but you can easily join it or
import fields.

```rust
let basket_items_subselect = basket_items
    .clone();

let basket_items_subselect = basket_items_subselect
    .add_field_expr("basket_date", basket_items_subselect.get_ref("basket")?.get_field("date")?)
// (select date from basket where basket.id = basket_id) as basket_date


let basket_items_join = basket_items
    .clone()
    .join_ref("basket");

let basket_items_join = basket_items_join
    .add_field_expr("basket_date", basket_items_join.get_join("basket")?.get_field("date")?)
// select basket.date as basket_date ... join basket on basket.id = basket_id
```

Of course, once you wrap this into your custom Structs it becomes very beautiful and usable:

```rust
struct BasketSet {
    table: Table<Postgres>,
}
impl BasketSet {
    pub fn new(ds: Postgres) -> Self {
        let table = Table::new("basket", ds.clone())
            .add_field("date")
            .add_field_id("id")

            .has_many_cb("items", ||BasketItemSet::new(ds.clone()), "id", "basket_id")
            .add_field_cb("item_count", |t|t.ref_items().count())
            .add_field_cb("total", |t|t.ref_items().total_price())

            .add_cb_field("items", |t|t.ref_items())
        ;
        Self { table }
    }

    pub fn date(&self) -> &Field {
        self.get_field("date").unwrap()
    }

    pub fn ref_items(&self) -> &BasketItemSet {
        self.get_ref("items").unwrap()
    }

    pub fn items(&self) -> &Field {
        self.ref_field("items").unwrap()
    }
}
```

and BasketItem:

```rust
struct BasketItemSet {
    table: Table<Postgres>,
}
impl BasketItemSet {
    pub fn new(ds: Postgres) -> Self {
        let table = Table::new("basket_item", ds.clone())
            .add_field("basket_id")
            .add_field("product_id")
            .add_field("quantity")
            .has_one_cb("basket", ||BasketSet::new(ds.clone()), "basket_id", "id")
            .has_one_cb("product", ||ProductSet::new(ds.clone()), "product_id", "id")

            // Add some optional fields
            .add_field_cb("basket_date", |t|t.ref_basket().date())
            .add_field_cb("default_price", |t|t.ref_product().default_price());
            .add_field_cb("total_price", |t|expr!(
                "{} * {}",
                t.quantity(),
                t.ref_product().default_price()
            ));
        Self { table }
    }

    pub fn basket_id(&self) -> &Field {
        self.get_field("basket_id").unwrap()
    }

    pub fn product_id(&self) -> &Field {
        self.get_field("product_id").unwrap()
    }

    pub fn quantity(&self) -> &Field {
        self.get_field("quantity").unwrap()
    }
    pub fn total_price(&self) -> &Field {
        self.get_field("total_price").unwrap()
    }

    pub fn ref_basket(&self) -> &BasketSet {
        self.get_ref("basket").unwrap()
    }
    pub fn ref_product(&self) -> &ProductSet {
        self.get_ref("product").unwrap()
    }
}
```

Finally, putting this all together, here is a beautiful example of using a single select to
calculate basket total price.

```rust

#[derive(Serialize, Deserialize)]
struct Basket {
    item_count: i32,
    total: i32,
}

let my_basket = BasketSet::new(postgres.clone())
    .load(123)
    .into::<Basket>();

println!("Basket total: {} for {} items", my_basket.total_price, my_basket.item_count);
```

## Callback joins and fields

In some cases, it makes sense to define your data set as a join between several tables. When you
are performing a join, it will collect the fields from both tables together (even if they already
have made some joins before).

In other cases, you would want a join to be optional. This can only be done with a LEFT join,
because any other join would affect number of columns returned with/without, but a LEFT join is
fully optional (as long as it matches into unique field).

You may also define some callback fields. This is good for some complex calculations, that you
wouldn't want to always happen like with the `total` and `item_count`.

DORM relies on the entity you pass in and it's deserialization details to infer which fields
you want to load:

```rust
#[derive(Serialize, Deserialize)]
struct BasketDate {
    date: Date,
}

let my_basket_totals = BasketSet::new(postgres.clone())
    .with_id(123)
    .into::<Basket>();

let my_basket_date = BasketSet::new(postgres.clone())
    .with_id(123)
    .into::<BasketDate>();

```

DataSet's into() function will determine which fields to load based on the type you are converting
into and the line above `with_id(123)` will infer the conditoin for loading the record. The code
above will generate 2 different queries - one for the totals and another for the date.

You may use as many different Structs as you want, there is no explicit bindings a transient
Model struct and the DataSet.

## Nested data structures

`Table::add_field_cb` is a powerful tool to implement various kinds of fields. Field may define a
component of a Query , so when DORM is fetching Model data it would amend the query in anticipation,
alternatively it may result in another DataSet.

If DataSet is returned and it cannot be easily incorporated into the main query, DORM will perform
a separate query to fill in the result:

```rust
#[derive(Serialize, Deserialize)]
struct BasketItems {
    date: Date,
    items: Vec<BasketItem>,
}

let my_basket_items = BasketSet::new(postgres.clone())
    .with_id(123)
    .into::<BasketItems>();
```
In this case DORM will look into the `items` field, which is a callback and returns a
BasketItemSet. To fill in the Vec of basket items, additional query will be performed.

I want also to point out that the ref_items() will produce a secondary DataSet which
has the same DataSource.

If the DataSource is identical (cloned Arc of the same instance), that means we are
querying from the same database. But potentially the related DataSet could be linked
to a different DataSource.

This powerful abstraction makes it possible for a model to fetch data from multiple
physical databases, sources or APIs and combine them seamlessly.

## Deleting, Inserting or Updating items

So far we have been using a pretty bare-bones structures for a model. We are able to
load data into a struct. There are also ways to store data or delete rows.

```rust
BasketSet::new(postgres.clone())
    .with_id(123)
    .delete();

let new_basket_id = BasketSet::new(postgres.clone())
    .insert(my_basket_items);
```

Methods `delete()` and `insert()` are can be added to your DataSet structure by a macro but
would look like this:

```rust
impl BasketSet {
    pub fn delete(&self) {
        self.table.delete();
    }

    pub fn insert(&self, item: impl Serialise) -> Value {
        self.table.insert(item)
    }
}
```

Method `update()` is also there, but you must not forget to specify `with_id()` before using it,
otherwise it may affect multiple rows.

Note: See also a map() method that can be used to individually update multiple rows in a DataSet.
It is an expensive operation though, and will result in multiple updates.

This brings us to the need of having a primary key inside Models.

## Persistence-aware Models

A pesistence-awer model has a field refering to it's data source and a key. This makes
it easier for developer to load a record, modify and then store it back.

```rust
#[derive(Serialize, Deserialize)]
#[dorm_model]
pub struct Basket {
    // _ds: (DataSource, id)
    date: Date,
    item_count: Option<i32>,
    total: Option<i32>,
    items: Vec<BasketItem>,
}
```

The `_ds` field is a special field that is used to store the DataSource, key and an initial
values of the record from the database.

```rust
let mut my_basket = Basket::load(BasketSet::new(ds.clone()), basket_id);

my_basket.date = Date::now();
my_basket.items.push(new_item);
let res = my_basket.save().await;
```

When record is being saved back, only the fields you have modified will be updated. Additionally
the data can only be saved into the original DataSet, if you modify some essential fields, that
would make it go outside of the DataSet, this will result in an error:

```rust
let mut my_basket = Basket::load(BasketSet::new(ds.clone()), basket_id);

my_basket.shipped = true;
let res = my_basket.save_into(ShippedBasketSet::new(ds.clone())).await;
```

This provides an essential guardrails for business application to avoid accidental logical errors.

## Validation and Sanitization

DORM does not offer a way for validation and sanitization of the data, but we recommend using
"nutype". Make sure that Serde is able to serialize your data structure and it will work fine
with DORM.


## Making DataSet static

You may see a problem with this code:

```rust
let expensive_products = ProductSet::new(postgres.clone());
let expensive_products = expensive_products
    .add_condition(expensive_products.default_price().gt(100));
```

The problem here is that add_condition takes `self` and this does not allow default_price() to
be borrow same object. This is a common problem with Rust. There are two ways to solve it.

First lets create condition in a temporary variable:
```rust
let expensive_products = ProductSet::new(postgres.clone());
let condition = expensive_products.default_price().gt(100);
let expensive_products = expensive_products
    .add_condition(condition);
```

This works now fine, but code readability suffers a lot. Let me show you a better way, that
also speeds things up a little. This relies on Lazy Statics.

First, lets rebuild our ProductSet to be static:

```rust
use std::sync::OnceLock;
use dorm::prelude::*;
use crate::postgres;

pub struct ProductSet {
    table: Table<Postgres>,
}

impl ProductSet {
    pub fn new() -> Table<Postgres> {
        ProductSet::table().clone()
    }
    pub fn table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        let table = TABLE.get_or_init(|| {
            Table::new("product", ds)
                .add_field("name")
                .add_field("description")
                .add_field("default_price");
        });

        table
    }
    pub fn name() -> &'static Field {
        ProductSet::table().get_field("name")
    }
    pub fn profit_margin() -> &'static Field {
        ProductSet::table().get_field("profit_margin")
    }
}
```

This implementation allows us to use ProducSet::name() without creating a new instance of
ProductSet. Next lets look how `fn main()` needs to change to provide postgres() connection:

```rust
static POSTGRESS: OnceLock<Postgres> = OnceLock::new();

pub fn postgres() -> Postgres {
    POSTGRESS
        .get()
        .expect("Postgres has not been initialized")
        .clone()
}

#[tokio::main]
async fn main() -> Result<()> {
    let (client, connection) = tokio_postgres::connect("host=localhost dbname=postgres", NoTls)
        .await
        .context("Failed to connect to Postgres")?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    POSTGRESS
        .set(Postgres::new(Arc::new(Box::new(client))))
        .map_err(|_| anyhow::anyhow!("Failed to set Postgres instance"))?;
}
```

Finally - here is the code to create expensive product set once again:

```rust
let expensive_products = ProductSet::new().add_condition(ProductSet::default_price().gt(100));
```

In fact you can move that into a method of a ProductSet:

```rust
impl ProductSet {
    pub fn expensive() -> Table<Postgres> {
        ProductSet::new().add_condition(ProductSet::default_price().gt(100))
    }
}
```

And your code will look like this:

```rust
let expensive_products = ProductSet::expensive();
```
