use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub fn create_connection_pool() -> r2d2::Pool<ConnectionManager<MysqlConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
