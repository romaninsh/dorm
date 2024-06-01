MVP:

0.1.0: Query Building
- [x] create a basic query type
- [x] have query ability to render into a SQL query
- [x] add ability to have expressions
- [x] add ability to have where conditions
- [x] add support for datasource
- [x] add support for dataset
- [x] add integration with sqlite
- [x] add integration with postgres
- [x] implement insert query
- [x] implement delete query
- [x] implement operations: (field.eq(otherfield))
- [x] implement parametric queries

0.2.0: Nested Query Building
- [x] properly handle nested queries
- [x] table should own DataSource, which should be cloneable and use Arc for client
- [x] implemented condition chaining
- [x] implemented and/or conditions
- [x] implemented expression query
- [x] implemented table::sum()
- [x] implemented TableDelegate trait
- [x] implemented Query::add_join()

0.3.0: Table Structure
- [x] add uniq id vendor
- [x] implemented Table::join_table() for merging tables
- [x] field prefixing with table alias/name (optional)
- [x] Table::join_table can be used to reference fields. Also add Table::with_join()
- [x] Table::join_table should preserve joins on other_table
- [x] When joining table, combine their UniqueIdVendors into one
- [x] Implement has_one and has_many in a lazy way
- [x] Implement expressions in a lazy way

0.4.0: Control field queries
- [ ] Implement a way to create a query with custom field references
- [ ] Implement a way to query with a serialized structure
- [ ] Separate fields from active fields structure
- [ ] Implement ability to specify which fields to query for

- [ ] add tests for all CRUD operations (ID-less table)
- [ ] implemented `each` functionality for DataSet
- [ ] implement functions: (concat(field, " ", otherfield))
- [ ] move postgres integration tests into a separate test-suite
- [ ] add tests for table conditions (add_condition(field1.eq(field2))
- [ ] implement sub-library for datasource, supporting serde
- [ ] add second data-source (csv) as an example
- [x] datasource should convert query into result (traited)
- [ ] select where a field is a sub-query
- [ ] insert where a field value is an expression
- [ ] insert where a field is a sub-query
- [x] select from a subquery
- [ ] add sql table as a dataset at a query level (+ clean up method naming)
- [ ] postgres expressions should add type annotation into query ("$1::text")

Pratcitacl tests:
- [x] Populate bakery tests
- [ ] Make bakery model more usable
- [ ] table.itsert_query should quote field names (bug)

Lazy features:
- [ ] Implement join_table_lazy()
- [ ] Implement add_field_lazy()


Minor Cases:
- [ ] Table::join_table should preserve conditions on other_table
- [ ] Table::join_table should resolve clashes in table aliases
- [ ] Condition::or() shouldn't be limited to only two arguments
- [ ] It should not be possible to change table alias, after ownership of Fields is given

Implementing examples:

- [x] Add query filters
- [x] Add sum() function

```rust
let vip_client = Table::new('client', db)
    .add_title('name')
    .add_field('is_vip')
    .add_condition('is_vip', true);

let sum = vip_client.sum('total_spent');
```

- [ ] Implement relations between tables

```rust
let mut clients = Table::new('client', db)
    .add_title('name')
    .add_field('is_vip');
let mut orders = Table::new('orders', db)
    .add_field('total');

users.has_many('orders', orders, 'order_id', 'id');

let vip_total = clients.clone()
    .add_condition('is_vip', true)
    .ref('orders')
    .sum('total');
```

- [ ] Implement syntax sugar for models
- [ ] Implement support for types

```rust

#[dorm::table]
struct Client {
    name: String,
    is_vip: bool,
}

#[dorm::table]
struct Order {
    #[dorm::has_one(Client, "id"))]
    user_id: i32,
    total: f64,
}

let vip_total = Client::new(db)
    .add_condition(is_vip.eq(true))
    .ref_orders()
    .sum(total);
```

# Future features

## Implement persistence-aware model

By a model we call a struct implementing ser/de traits that can be used with
DataSet to load, store and iterate over data. We do not need a basic implementation
to be persistence-aware. However with persistence-aware model we can implement
id-tracked conditioning. The model will know where it was loaded from and
will be able to update itself if changed, which can even be done on drop.

```rust
#[dorm::persistence(id = "my_id")]
struct Client {
    my_id: i32,
    name: String,
    is_vip: bool,

    _dsp: DataSourcePersistence,  // required for persistence-aware model
}

let client = ClientSet::new(db)
    .load(1);

db.transaction(|_| {

    client.orders.each(|order: Order| {
        order.price-= 10;
    });

    client.is_vip = true;
    client.save();
});
```

## Implement non-table SQL data source

Basic implementation allows to use Table as an ORM data source. We can implement
a read-only source that have a query as a source.

TODO: query-based model can be a curious feature, but this example should be rewritten
to use a different table-like construct, returned by table.group() method.

```rust
struct GraphData {
    date: Date,
    value: f64,
}

struct DailyDeployments {
    table_deployment: Deployments,
    query: Query,
}

impl DailyDeployments {
    // like Deployments, but with date grouping and date-range
    pub fn new(ds: DataSource, date_from: Date, date_to: Date) -> Self {
        let td = Deployments::new(ds);
        let query = td
            .query_fields(vec![td.date(), td.value()])
            .add_condition(td.date().gte(date_from))
            .add_condition(td.date().lte(date_to))
            .group_by(td.date());

        Self { ds, table }
    }
    pub fn date(&self) -> Field {
        self.query.field(0)
    }
}

let dd = DailyDeployments::new(db, Date::new(2020, 1, 1), Date::new(2020, 1, 31));
let data = dd.query().fetch::<GraphData>();
```

## Implement cross-datasource operations

Developers who operate with the models do not have to be aware of the data source.
If you want to implement this, then you can define your data sets to rely on
factories for the data-set:

```rust
let client_set = ClientSet::factory();
let client = client_set.load_by_auth(auth_token)?;
let basket = client.basket();  // basket is stored in session/memory
for item in basket.items()?.into_iter() {
    let stock = item.stock();
    stock.amount -= item.amount;
    stock.save();  // saves into SQL

    item.status = "sold";
    item.save();   // item data is stored in cache
}
basket.archive();  // stores archived basked into BigQuery
```

## Implement in-memory cache concept

This allows to create in-memory cache of a dataset. Finding a record
in a cache is faster. Cache will automatically invalidate items if
they happen to change in the database, if the datasource allows
subscription for the changes. There can also be other invalidation
mechanics.

Cache provides a transparent layer, so that the business logic code
would not be affected.

```rust
let client_set = ClientSet::new(ClientCache::new(postgres));
// use client_set as usual
```
