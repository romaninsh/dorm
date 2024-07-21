use std::sync::OnceLock;

use dorm::prelude::Postgres;

pub mod baker;
pub mod bakery;
pub mod cake;
pub mod cakes_bakers;
pub mod customer;

pub mod lineitem;
pub mod order;

pub use baker::BakerSet;
pub use bakery::BakerySet;

static POSTGRESS: OnceLock<Postgres> = OnceLock::new();

pub async fn create_tables() {
    let p = postgres();
    let conn = p.client();
    conn.execute(bakery::BakerySet::create(), &[])
        .await
        .unwrap();
    conn.execute(
        "insert into bakery values (1, 'Poor Bakery', 7), (2, 'Profitable Bakery', 15)",
        &[],
    )
    .await
    .unwrap();
    conn.execute(baker::BakerSet::create(), &[]).await.unwrap();
    conn.execute(cake::CakeSet::create(), &[]).await.unwrap();
    conn.execute(customer::CustomerSet::create(), &[])
        .await
        .unwrap();
    conn.execute(order::OrderSet::create(), &[]).await.unwrap();
    conn.execute(lineitem::LineitemSet::create(), &[])
        .await
        .unwrap();
}

pub fn set_postgres(postgres: Postgres) {
    POSTGRESS.set(postgres).unwrap();
}

pub fn postgres() -> Postgres {
    POSTGRESS
        .get()
        .expect("Postgres has not been initialized")
        .clone()
}
