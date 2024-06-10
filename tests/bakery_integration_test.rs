use std::{
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use dorm::prelude::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use testcontainers::{runners::AsyncRunner, ContainerAsync};

use testcontainers_modules;
use tokio_postgres::NoTls;

static POSTGRESS: OnceLock<Postgres> = OnceLock::new();
static CONTAINER: OnceLock<ContainerAsync<testcontainers_modules::postgres::Postgres>> =
    OnceLock::new();

async fn start_postgres() -> Result<()> {
    let pg_container = testcontainers_modules::postgres::Postgres::default()
        .with_host_auth()
        .start()
        .await
        .context("Failed to start Postgres container")?;

    let connection_string = &format!(
        "postgres://postgres@{}:{}/postgres",
        pg_container.get_host().await?,
        pg_container.get_host_port_ipv4(5432).await?
    );
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

#[tokio::test]
async fn should_create_bucket() {
    start_postgres().await.context("starting postgres").unwrap();

    let postgres = POSTGRESS.get().unwrap();

    let res = postgres
        .query_raw(&Query::new().add_column("result".to_string(), expr!("1 + 1")))
        .await
        .unwrap();

    assert_eq!(res, vec![json!({"result": 2})]);
}
