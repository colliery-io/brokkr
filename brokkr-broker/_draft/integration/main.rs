#![cfg(test)]
#[path = "../common.rs"]
mod common;
mod dao;

use brokkr_models::establish_connection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Integer;
use diesel::QueryableByName;
use dotenv::dotenv;
use std::env;

#[derive(QueryableByName, PartialEq, Debug)]
struct OneRow {
    #[diesel(sql_type = Integer)]
    value: i32,
}

#[test]
fn test_establish_connection() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut connection = establish_connection(database_url.clone());

    let result = sql_query("SELECT 1 AS value").get_result::<OneRow>(&mut connection);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, 1);
}
