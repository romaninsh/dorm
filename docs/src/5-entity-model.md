# Entity Model

So we need to conveniently create a `product` table objects,
but we don't want to re-populate all the fields and conditions
every time.

DORM recommends you to have a object called `Product` that
would vend `Table` objects. We will place the code inside
a `model` subfolder.

While we are building our `Product` type, lets also create
a static instance of our DataSource:

inside your model/mod.rs:

```rust
use dorm::prelude::Postgres;

pub mod products;
pub use products::*;

static POSTGRESS: OnceLock<Postgres> = OnceLock::new();

pub fn set_postgres(postgres: Postgres) -> Result<()> {
    POSTGRESS
        .set(postgres)
        .map_err(|_| anyhow::anyhow!("Failed to set Postgres instance"))
}

pub fn postgres() -> Postgres {
    POSTGRESS
        .get()
        .expect("Postgres has not been initialized")
        .clone()
}
```

Now you would need to call set_postgress() when your
tokio_postgress client is ready and you can import and
call postgress() from your models.

Lets create file `models/products.rs`:

```rust
use dorm::prelude::*;
use crate::postgres;

pub struct Product {}
impl Product {
    pub fn table() -> Table<Postgres> {
        Product::static_table().clone()
    }
    pub fn static_table() -> &'static Table<Postgres> {
        static TABLE: OnceLock<Table<Postgres>> = OnceLock::new();

        TABLE.get_or_init(|| {
            Table::new("product", postgres())
                .with_id_field("id")
                .with_field("name")
        })
    }

    pub fn name() -> Arc<Field> {
        Product::static_table().get_field("name").unwrap()
    }
}
```

Now that you have created a Product type, we can reference
it in your application like this:

```rust
use model;

let products = model::Product::table();
writeln!(products.get_select_query().preview());

// renders into: SELECT id, name FROM product

let low_cal_products = model::Product::table()
    .with_condition(
        model::Products::calories().lt(100)
    );
writeln!(low_cal_products.get_select_query().preview());

// renders into: SELECT id, name, calories FROM product WHERE (calories < 100)
```

This is already much more portable, but we can do better.
Add this to your `model/products.rs` and

```rust
pub struct LowCalProduct {}
impl LowCalProduct {
    pub fn table() -> Table<Postgres> {
        Product::table().with_condition(
            Product::calories().lt(100)
        )
    }
}
```

You can addopt a different approach here, those are just
a few recommendations. Later we will explore a way to
create a dynamic business entity pattern, but now
we will focus on something more fun - joins.
