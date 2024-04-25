DataSet API

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
- [ ] implement functions: (concat(field, " ", otherfield))
- [ ] implement type constraints for expressions and fields
- [ ] add tests for table conditions (add_condition(field1.eq(field2))
- [x] implement parametric queries
- [ ] implement sub-library for datasource, supporting serde
- [x] datasource should convert query into result (traited)
- [ ] select where a field is a sub-query
- [ ] insert where a field value is an expression
- [ ] insert where a field is a sub-query
- [ ] select from a subquery
- [ ] add sql table as a dataset

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
