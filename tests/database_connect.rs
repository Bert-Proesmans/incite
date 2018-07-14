extern crate diesel;
extern crate dotenv;
extern crate failure;
extern crate incite;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

#[test]
fn diesel_connect() -> Result<(), failure::Error> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "file:server.db".into());
    let _connection = SqliteConnection::establish(&db_url)?;

    Ok(())
}
