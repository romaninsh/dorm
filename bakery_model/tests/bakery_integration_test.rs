use std::{
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use dorm::prelude::*;
use serde_json::json;
use tokio_postgres::NoTls;

use bakery_model::{self, connect_postgres, postgres};

// async fn create_bootstrap_db() -> Result<()> {
//     let client = POSTGRESS.get().unwrap().client();

//     bakery_model::create_tables().await;

//     client
//         .execute(
//             "
//         CREATE TABLE IF NOT EXISTS table1 (
//                     id SERIAL PRIMARY KEY,
//                     name TEXT NOT NULL
//                 );
//                 ",
//             &[],
//         )
//         .await?;
//     client
//         .execute(
//             "
//                 CREATE TABLE IF NOT EXISTS table2 (
//                     id SERIAL PRIMARY KEY,
//                     data TEXT NOT NULL
//                 );",
//             &[],
//         )
//         .await?;

//     client
//         .execute(
//             "
//             INSERT INTO table1 (name) VALUES ('Alice'), ('Bob');
//             ",
//             &[],
//         )
//         .await?;
//     client
//         .execute(
//             "
//             INSERT INTO table2 (data) VALUES ('Data1'), ('Data2');
//             ",
//             &[],
//         )
//         .await?;

//     Ok(())
// }

async fn init() {
    connect_postgres()
        .await
        .context("starting postgres")
        .unwrap();
}

// TODO: get rid of testcontainers, yukk.

#[tokio::test]
async fn should_create_bucket() {
    init().await;

    let postgres = postgres();

    let res = postgres
        .query_raw(
            &Query::new()
                .set_table("table1", None)
                .add_column_field("name"),
        )
        .await
        .unwrap();

    assert_eq!(res, vec![json!({"name": "Alice"}), json!({"name": "Bob"}),]);
}

// #[tokio::test]
// async fn test_bakery() {
//     init().await;

//     let product_set = bakery_model::BakerySet::new()
//         .with_condition(bakery_model::BakerySet::profit_margin().gt(10));

//     let postgres = POSTGRESS.get().unwrap();
//     let res = postgres
//         .query_opt(&product_set.get_select_query())
//         .await
//         .unwrap()
//         .unwrap();

//     assert_eq!(
//         res,
//         json!({
//             "name": "Profitable Bakery",
//             "profit_margin": 15,
//         })
//     );
// }
