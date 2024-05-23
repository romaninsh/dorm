use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use dorm::prelude::*;
use tokio_postgres::NoTls;

use crate::model::bakery::BakerySet;

mod model;
extern crate dorm;

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

    let product_set = BakerySet::new().with_condition(BakerySet::profit_margin().gt(10));

    println!("Hello world Bakery");
    Ok(())
}
