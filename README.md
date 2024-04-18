# DORM - the Dry Object-Relational Mapper

While most ORM libraries are designed to work specifically with the data by proactively
fetching it, DORM prefers to be lazy and focus on query-mapping and lazy hydration.

## Typical ORM code (Diesel):

Suppose we want to print out list of online users along with a total amount of their basket. Diesel
would require you to fetch lits of users and then iterate over the results potentially producing
many queries:

```rust
let connection = establish_connection();
let thirty_minutes_ago = Utc::now().naive_utc() - chrono::Duration::minutes(30);

let active_users = users
    .filter(last_online.gt(thirty_minutes_ago))
    .load::<User>(&connection)
    .expect("Error loading active users");

for user in active_users {
    let basket_total: f64 = orders
        .filter(user_id.eq(user.id))
        .filter(is_active.eq(false))
        .select(sum(amount))
        .first(&connection)
        .unwrap_or(0.0);

    println!("Active user {} has a basket worth ${:.0}", user.email, basket_total);
}
```

The DORM approach is different, where you can define a query and then iterate over the results. Queries are hydrated only when needed.

```rust
let datasource = dorm::DataSource::new::<Postgres>(establish_connection());
let thirty_minutes_ago = Utc::now().naive_utc() - chrono::Duration::minutes(30);

let orders = datasource.table("user")
    .filter("last_online", ">", thirty_minutes_ago)
    .ref_table("order")
    .filter("is_active", "=", false)

let results = orders
    .query_expr("CONCAT('Active user ', {user.email}, 'has a basket worth ', sum({amount}))")
    .iter_col()

for result in results {
    println!("{}", result);
}
```

DORM also provides syntatic sugar by creating data models for you. Using those
your code would look like this:

```rust
let users = User::new(datasource);

let orders = users
    .filter(users.last_online().gt(thirty_minutes_ago))
    .ref_orders();

let results = orders
    .filter(orders.is_active().is(false))
    .query_expr(Expression::concat(
        'Active user ',
        user.email(),
        'has a basket worth ',
        Expression::sum(orders.amount()
    )).iter_col()
```

## DORM dive

Startign with the basics, lets build entire app and solve number of challenges.

### 1. Add support for datasource

DORM relies on 3rd party libraries to connect to a database, therefore no extra steps
are necessary. You may also integrate your own datasource by implementing a `DataSource` trait.

### 2. Define data entities

Business entities will shadow either a `Table` or a compatible struct. The table is the
most obvious, but it can also be a `UnionQuery` or a `StoredFunctionCall` or even an `Expression`.

You can deliver a bunch of boilerplate code by using a `Table`, which might not be available
if you use a more exotic data source.

Unlike Diesel, DORM focuses on modeling DataSets and not the data itself. Here is an example

```rust
// model/client.rs
struct Client {
    id: Option(i32),
    name: String,
    is_vip: bool,
}

struct Clients <'a> {
    type Item = Client;

    // We can be specific about ds type here. Reference is not owned but must outlive us.
    ds: &'a PostgresDataSource,

    // Table is owned and must outlive any queries that would be generated.
    table: Table,
}

impl Clients {
    fn new(ds: &'a PostgresDataSource) -> Self {

        // We specify the table name here and structure
        let table = Table::new("client", ds)
            .add_title("name")
            .add_field("is_vip")
        ;

        Self { ds, table }
    }

    fn name(&self) -> &Field {
        self.table.field("name")
    }

    fn is_vip(&self) -> &Field {
        self.table.field("is_vip")
    }

    fn ref_orders(&self) -> Order {
        self.table.ref_many(Order::new(self.ds), "client_id")
    }
}

// Gives us more boilerplate methods
impl PostgresTableDataSet for Clients {
}
```

NOTE: the model can be initially generated, but it then becomes a part of your codebase
and can have additional business-level operations added to it.

### 3. Cross-dataset use

DORM allows you to define data relationship across data-sets. As an example, our `Order` model
will feature integration with `DeliveryUpdate` model that is defined by a third party API.

```rust
// model/delivery_update.rs
struct DeliveryUpdate {
    date: DateTime<Utc>,
    order_id: i32,
    status: String,
}

struct DeliveryUpdates <'a> {
    type Item = DeliveryUpdate;

    order_id: i32,
}

impl DeliveryUpdates {
    fn new(order_id: i32) -> Self {
        Self { order_id }
    }
}

impl CustomDataSet for DeliveryUpdates {

    // Will fetch data from the API and return
    fn iter(&self) -> Box<dyn Iterator<Item = Self::Item>> {
        todo!()
    }

    // TBC:
}
```

Next we can connect up the `Order` model with the `DeliveryUpdate` model.

```rust
# model/order.rs
struct Order {
    id: Option(i32),
    // note the absence of a client_id field here
    is_shipped: bool,
    amount: f64,
}
struct Orders <'a> {
    type Item = Order;

    ds: &'a PostgresDataSource,
    table: Table,
}

impl Orders {
    fn new(ds: &'a PostgresDataSource) -> Self {
        let table = Table::new("order", ds)
            .add_field("client_id")
            .add_field("is_shipped")
            .add_field("amount")
        ;

        Self { ds, table }
    }

    fn client_id(&self) -> &Field {
        self.table.field("client_id")
    }

    fn amount(&self) -> &Field {
        self.table.field("amount")
    }

    fn is_shipped(&self) -> &Field {
        self.table.field("is_shipped")
    }

    fn filter_shipped(self) -> Self {
        self.table.filter(is_shipped().is(true))
    }

    fn ref_delivery_updates(&self) -> DeliveryUpdates {
        // field.into::<i32>() will automatically hydrate/execute query
        // but will error if execution results in more than one row
        DeliveryUpdates::new(self.table.field("id"))
    }

    // This allows us to traverse back into Client
    fn ref_client(&self) -> Client {
        self.table.ref_one(Client::new(self.ds), client_id())
    }
}
```

### 4. Refactor order to have `Lines`

Currently our `Order` model contains the entire amount of the order, but lets say
that we have recived a new requriement to split the order into lines. With DORM
the refactoring is easy. `Order` will not be changed, however the underlying
DataSet will be. This is one distinctive feature of DORM, where the DataSet and
the model are not tightly coupled.

We will introduce one more concept here - `product_id` column would not make sense
to us, so we will use a `Product` reference. Line does not own the product, it
merely references it.

```rust
// model/line.rs
struct Line <'a> {
    id: Option(i32),
    product: &'a Product,
    quantity: i32,
    price: f64,
}

impl Line {
    fn new(product: &'a Product, quantity: i32, price: f64) -> Self {
        Self { id:None, product, quantity, price }
    }

    // Use a custom method to load/save record into a row data
    fn load_from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            product: Product::from_id(row.get("product_id")),
            quantity: row.get("quantity"),
            price: row.get("price"),
        }
    }
    fn save_to_row(&self, row: &mut Row) {
        row.set_id(self.id);
        row.set("product_id", self.product.id);
        row.set("quantity", self.quantity);
        row.set("price", self.price);
    }
}

struct Lines <'a> {
    type Item = Line;

    ds: &'a PostgresDataSource,
    order: &'a Order,
    table: Table,
}

// Lines can only be used in the context of an Order
impl Lines {
    fn new(ds: &'a PostgresDataSource, order: &'a Order) -> Self {
        let table = Table::new("line", ds)

            // where order_id in (select id from order ..)
            .filter(order_id().in(order.only_ids()))

            .add_field("product_id")
            .add_field("quantity")
            .add_field("price");
        let table = table
            .add_field_expr("amount", table.expr("{quantity} * {price}"))
        ;

        Self { ds, order, table }
    }

    fn product_id(&self) -> &Field {
        self.table.field("product_id")
    }

    fn quantity(&self) -> &Field {
        self.table.field("quantity")
    }

    fn price(&self) -> &Field {
        self.table.field("price")
    }

    fn amount(&self) -> &Field {
        self.table.field("amount")
    }

    fn ref_order(&self) -> Order {
        self.order
    }

    fn ref_product(&self) -> Product {
        self.table.ref_one(Product::new(self.ds), product_id())
    }
}
```

to finish our refactoring, we need to update `Order` slightly:

- should have a `ref_lines` method
- no longer neds `amount` field

```rust
impl Orders {
    fn new(ds: &'a PostgresDataSource) -> Self {
        let lines = Lines::new(ds, self);

        let table = Table::new("order", ds)
            .add_field("client_id")
            .add_field("is_shipped")
            .add_expression("amount", lines.sum(lines.amount()))
        //.add_field("amount")
        ;

        Self { ds, table }
    }

    fn client_id(&self) -> &Field {
        self.table.field("client_id")
    }

    fn amount(&self) -> &Field {
        self.table.field("amount")
    }

    fn is_shipped(&self) -> &Field {
        self.table.field("is_shipped")
    }

    fn filter_shipped(self) -> Self {
        self.table.filter(is_shipped().is(true))
    }

    fn ref_delivery_updates(&self) -> DeliveryUpdates {
        // field.into::<i32>() will automatically hydrate/execute query
        // but will error if execution results in more than one row
        DeliveryUpdates::new(self.table.field("id"))
    }

    // This allows us to traverse back into Client
    fn ref_client(&self) -> Client {
        self.table.ref_one(Client::new(self.ds), client_id())
    }
}
```

As you can see here - we have kept the `amount` field in the `Order` model and it
still shows a total sum of all the order lines. The decoupling of the model and
the DataSet allows us to make changes to the DataSet without affecting wider business
logic.

### 5. Querying and Async

Now comes a fun part, once DataSet and models are defined, we can perform wide range
of operations.

```rust
let total_orders = Client::new(db)
    .ref_orders()
    .filter_shipped()
    .sum(Order::ammount).await?;
```

In here we are addressing orders of all the clients that are shipped and summing up the
ammount. The resulting query for this would be:

```sql
select sum(amount) from order where is_shipped = true and client_id in (select id from client)
```

Next lets look at how we can add a new order:

```rust
let clients = Clients::new(db)
clients.filter(clients.name().like("Pear Company"))
    .ref_orders()
    .save_and_filter(Order {
        client_id: 1,
        is_shipped: false,
        amount: 100.0,
    }).await?
    // Save will set the scope of the dataset to a newly added row
    .ref_lines()
    // Will automatically infer order
    .import(vec![
        Line { id: None, product_id: Product::new(db).with_name("Table").id(), quantity: 3,  price: 10.0 },
        Line { id: None, product_id: Product::new(db).with_name("Chair").id(), quantity: 10,  price: 15.0 },
    ]).await?;
```

Note: all the query-building stuff is sync, because it does not require any operations
with the database. Data fetching will either be async or return iterators.

### 6. Updating records

Why not introduce a class for `OrderWithLines` and make that work instead?

```rust

// TODO: confusion here!!!!!!!!!

struct OrderWithLines {
    order: Order,
    lines: Vec<Line>,
}

impl Orders {
    async fn save_with_lines(&self, order_with_lines: OrderWithLines) -> Result<(OrderWithLines)> {
        let mut order = order_with_lines.order;
        let lines = order_with_lines.lines;

        // will update order order_id
        self.save_mut(order).await?;

        self.filter_id(order.id).ref_lines().import(lines).await;
        Ok(order_with_lines)
    }
}
```

It's worth noting that save_mut() will mutate the order

A typical way to update a record, is to load it, modify and then save it. This is how
it can be done in our case:

```rust
let order_id = 1;

let mut order = Orders::new(db).filter_id(order_id).load(Order::new());
order.is_shipped = true;
Orders::new(db).save(&order)?;
```

It looks like this code can be shortened:

```rust
let updated_orders = Orders::new(db)
    .filter_id(order_id)
    .map(|order|order.is_shipped=true).await?;
```

This code performs the update and then still returns the record. This is because

All save methods will create a new record if id is None, otherwise it will update
the record with a corresponding ID, failing if record does not exist or cannod be
inserted/updated.

- `save` - saves the record returning Promise<Result(self)>, `order.save(rec).await?`
- `save_and_get_id` - saves and returns the id of the record
- `save_mut` - saves the record, reloads it back and record by filling in `id` and
  any calculated fields
- `reload` - provided with a record with id, will reload it from database
- `save_and_filter` - saves the record and sets the scope of the dataset to the newly
  added record
- `import_and_filter` - adds multiple records and sets scope to newly added records
- `replace` - adds multiple record replacing existing one (if they exist)

### ID field and title field

Each table defaults to having an `id` field and a `name` field. The `id` field is required
to perform save/reload/replace etc, but is not essential for querying. You can change id
field with `table.with_id("order_id")`. You do not need to add ID as a field.

The title field is used for display purposes. You can appoint any field with `table.add_title()`,
however this field is identical to `add_field` in any way.

// TODO - not sure if title fields are needed

### 7. Left Joins and composing DataSets

For us left join is special. It allows us to fetch additional data without disrupting
unique IDs. Additionally I'm going to show you how to create a DataSet which is built
on an existing DataSet.

For this lets pretent that VIP clients have a separate table `client_vip` which contains
two fields: `client_id` and `vip_level`. This table will have no unique ID.

```rust
struct ClientVip {
    id: Option(i32),
    name: String,
    vip_level: i32,
}

struct ClientVipSet {
    ds: DataSet,
    client_table: Table,
    vip_join: Join,
    table: Table,
}

impl ClientVipSet {
    fn new(ds: DataSet) -> Self {

        // Create client only to take it's table
        let client_table = Client::new(ds).table;

        // Add filter for our own table
        let table = client_table.clone()
            .add_filter(table.field("is_vip").true());

        // Add left join and a field
        let vip_join = table
            .join("client_vip", "client_id", table.field("client_id"))
            .add_field("vip_level");

        Self { ds, table, client_table, vip_join }
    }

    fn client_id(&self) -> &Field {
        self.table.field("client_id")
    }

    fn vip_level(&self) -> &Field {
        self.table.field("vip_level")
    }

    fn ref_client(&self) -> Client {
        self.table.ref_one(Client::new(self.ds), client_id())
    }
}
```

This is how you can use it:

```rust

let vip_clients = ClientVipSet::new(db);

// Add a new VIP client eventually, we do not care about the ID (depending on your runtime)
tokio::spawn(async move {
    vip_clients
        .insert(ClientVip { id: None, name: "Pear Company", vip_level: 1 })
        .await
        .unwrap();
});
```

The insert operation will actually require a client_id and will use it to insert a record
into the `client_vip` table.

You may notice that we've kept `client_table` dataset, and here is how it can be useful
to us:

```rust
impl ClientVipSet {
    async fn promote_client_to_vip(&self, client_id: i32, vip_level: i32) -> Result<()> {

        self.ds.transaction(async {
            let client_id = self.client_table
                .filter_id(client_id)
                .map(|client| client.is_vip = true; client)
                .await?.id;

            self.vip_join.query().insert(
                Row::new()
                    .set("client_id", client_id)
                    .set("vip_level", vip_level)
            ).await?;
        }).await
    }

    async fn demote_vip_client(&self, client_id: i32) -> Result<()> {

        self.ds.transaction(async {
            // Confirm that client is a VIP
            self.load_by_id(client_id).await?;

            client_id = self.client_table.filter_id(client_id).load().await?.id;

            self.vip_join.query().filter("client_id", client_id).delete().await?;
        }).await
    }
}
```

Here we are also using transaction wrappers. Low-level operations may also use
transaction internally, but a wrapper is a great way to make sure that transactions
are rolled back properly. Here is a logic for transactions:

```rust

self.ds.transaction(async {
    do_thing_1().await?;

    self.ds.transaction(async {
        do_thing_2().await?;
    }).await?

    do_thing_3().await?;
}).await
```

- If thing 1 fails, it will be rolled back
- If thing 2 fails, it will be rolled back and thing 1 will be rolled back
- If thing 3 fails, everything will be rolled back

### 8. Possibly future features

#### Exploration of DataSet

Since DataSet is constructed dynamically, we might want to explore it, for example, to
build a UI structure or export schema.

The problem here is that DataSet represents a database structure, and it may be different
to the model structure.

#### Hooks

Hooks is a way to add additional checks, validations or value manipulation. For example,
you can add before_save hook to encrypt a password.

The problem with hooks is that it makes code structure non-transparent and less rust-ideomatic.

#### Linking model to DataSet

The idea here is to allow model to be saved back into DataSet directly:

```rust
let client = Clients::new(db).load_by_id(1).await?;
client.name = "New Name";
client.save().await?;
```

I believe that map() alows a much better way to perform the same thing:

```rust
let client = Clients::new(db).filter_by_id(1).map(|client| {
    client.name = "New Name";
    client
}).await?;
```

#### Console / REPL
