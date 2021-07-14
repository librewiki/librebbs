#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
pub mod db;
pub mod models;
pub mod routes;
pub mod schema;
use actix_cors::Cors;
use actix_web::{
    middleware::{Logger, NormalizePath},
    web, App, HttpServer,
};
use dotenv::dotenv;
use std::env;

pub async fn run() -> std::io::Result<()> {
    dotenv().ok();

    let pool = db::create_connection_pool();
    println!("http://{}", env::var("HOST").expect("HOST is not set"));

    HttpServer::new(move || {
        let cors_str = env::var("CORS").unwrap_or_default();
        let cors_split = cors_str.split("|");
        let mut cors = Cors::default();
        for domain in cors_split {
            cors = cors.allowed_origin(&domain);
        }

        App::new()
            .wrap(NormalizePath::default())
            .wrap(Logger::default())
            .wrap(cors)
            .data(pool.clone())
            .service(web::scope("/v1").service(routes::scope()))
            .service(routes::scope())
    })
    .bind(env::var("HOST").expect("HOST is not set"))?
    .run()
    .await
}
