use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use url::Url;

#[derive(Clone)]
pub struct ConnectionPool {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

pub fn create_shared_connection_pool(base_url: &str, database_name: &str, max_size: u32) -> ConnectionPool {
    let mut url = Url::parse(base_url).expect("Invalid base URL");
    url.set_path(database_name);

    let manager = ConnectionManager::<PgConnection>::new(url.as_str());

    let pool = Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create connection pool");

    ConnectionPool { pool }
}

