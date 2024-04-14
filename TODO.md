DataSet API

- [x] create a basic query type
- [x] have query ability to render into a SQL query
- [x] add ability to have expressions
- [ ] add ability to have where conditions
- [ ] add support for datasource
- [ ] add support for dataset
- [ ] add integration with sqlite
- [ ] add integration with postgres
- [ ] add sql table as a dataset

Implementing examples:

- [ ] Add query filters
- [ ] Add sum() function

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
