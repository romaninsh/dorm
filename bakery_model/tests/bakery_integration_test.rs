use std::{
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use dorm::prelude::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};

use testcontainers_modules;
use tokio_postgres::NoTls;

use bakery_model;

static POSTGRESS: OnceLock<Postgres> = OnceLock::new();
static CONTAINER: OnceLock<ContainerAsync<testcontainers_modules::postgres::Postgres>> =
    OnceLock::new();

pub fn postgres() -> Postgres {
    POSTGRESS
        .get()
        .expect("Postgres has not been initialized")
        .clone()
}

async fn start_postgres() -> Result<()> {
    let pg_container = testcontainers_modules::postgres::Postgres::default()
        .start()
        .await
        .context("Failed to start Postgres container")?;

    let connection_string = &format!(
        "postgres://postgres@{}:{}/postgres",
        pg_container.get_host().await?,
        pg_container.get_host_port_ipv4(5432).await?
    );

    dbg!(pg_container.get_host().await?);
    CONTAINER
        .set(pg_container)
        .map_err(|_| anyhow::anyhow!("Failed to store container reference"))?;

    let timeout = Duration::from_secs(30); // Max time to wait
    let start_time = Instant::now();
    let mut last_error: Result<()> = Ok(());

    while Instant::now().duration_since(start_time) < timeout {
        match tokio_postgres::connect(&connection_string, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("connection error: {}", e);
                    }
                });

                POSTGRESS
                    .set(Postgres::new(Arc::new(Box::new(client))))
                    .map_err(|_| anyhow::anyhow!("Failed to set Postgres instance"))?;

                bakery_model::set_postgres(POSTGRESS.get().unwrap().clone());

                println!("Successfully connected to the database.");
                return Ok(());
            }
            Err(e) => {
                println!("Error connecting to database: {}, retrying...", &e);
                last_error = Err(anyhow::Error::new(e));
                // sleep(Duration::from_secs(2)).await; // Wait before retrying
                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    last_error
}

async fn create_bootstrap_db() -> Result<()> {
    let client = POSTGRESS.get().unwrap().client();

    bakery_model::create_tables().await;

    client
        .execute(
            "
        CREATE TABLE IF NOT EXISTS table1 (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL
                );
                ",
            &[],
        )
        .await?;
    client
        .execute(
            "
                CREATE TABLE IF NOT EXISTS table2 (
                    id SERIAL PRIMARY KEY,
                    data TEXT NOT NULL
                );",
            &[],
        )
        .await?;

    client
        .execute(
            "
            INSERT INTO table1 (name) VALUES ('Alice'), ('Bob');
            ",
            &[],
        )
        .await?;
    client
        .execute(
            "
            INSERT INTO table2 (data) VALUES ('Data1'), ('Data2');
            ",
            &[],
        )
        .await?;

    Ok(())
}

async fn init() {
    start_postgres().await.context("starting postgres").unwrap();
    create_bootstrap_db().await.context("seeding db").unwrap();
}

// TODO: get rid of testcontainers, yukk.

// #[tokio::test]
// async fn should_create_bucket() {
//     init().await;

//     let postgres = POSTGRESS.get().unwrap();

//     let res = postgres
//         .query_raw(
//             &Query::new()
//                 .set_table("table1", None)
//                 .add_column_field("name"),
//         )
//         .await
//         .unwrap();

//     dbg!(&res);

//     assert_eq!(res, vec![json!({"name": "Alice"}), json!({"name": "Bob"}),]);
// }

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
