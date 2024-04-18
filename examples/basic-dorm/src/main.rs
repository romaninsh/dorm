use tokio_postgres::{Error, NoTls};

mod model;
extern crate dorm;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost dbname=postgres", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Hello from Basic Dorm example!");
    Ok(())
}
