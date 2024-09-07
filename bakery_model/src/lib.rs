use std::sync::OnceLock;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio_postgres::NoTls;

use dorm::prelude::Postgres;

pub mod bakery;
// pub mod cake;
// pub mod cakes_bakers;
pub mod client;
// pub mod customer;
pub mod product;

// pub mod lineitem;
pub mod order;

// pub use bakery::BakerySet;
// pub use product::ProductSet;

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

pub async fn connect_postgres() -> Result<()> {
    let connection_string = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgress@localhost:5432/postgres".to_string());

    let timeout = Duration::from_secs(3); // Max time to wait
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

                set_postgres(Postgres::new(Arc::new(Box::new(client))))?;

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
