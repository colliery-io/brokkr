use diesel::pg::PgConnection;
use diesel::prelude::*;
pub mod models;
pub mod schema;



#[allow(dead_code)]
/// This exists to manage migrations and some basic testing in this crate without a specific DAL in place. 
pub (crate) fn establish_connection(database_url: String) -> PgConnection {
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
