use std::sync::Arc;

use dorm::prelude::*;
use tokio_postgres::NoTls;

use anyhow::Result;

extern crate dorm;

#[tokio::main]
async fn main() -> Result<()> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost dbname=postgres", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let postgres = Postgres::new(Arc::new(Box::new(client)));

    // let github_authors_and_teams = Table::new("dx_teams", &postgres).add_field("source_id");
    // let dx_team_hierarchies =
    //     Table::new("dx_team_hierarchies", &postgres).add_field("anchestor_id");

    // let join = github_authors_and_teams.add_join(
    //     dx_team_hierarchies,
    //     github_authors_and_teams
    //         .get_field("id")
    //         .eq(dx_team_hierarchies.get_field("anchestor_id")),
    // );

    Ok(())
}
