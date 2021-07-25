#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
pub mod auth;
pub mod connection_info;
pub mod custom_error;
pub mod db;
pub mod models;
pub mod routes;
pub mod s3;
pub mod schema;
use actix_cors::Cors;
use actix_web::{
    middleware::{DefaultHeaders, Logger},
    web, App, HttpServer,
};
use dotenv::dotenv;
use std::env;

pub async fn run() -> std::io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let _guard = sentry::init((
        env::var("SENTRY_URL").expect("SENTRY_URL is not set"),
        sentry::ClientOptions {
            auto_session_tracking: true,
            session_mode: sentry::SessionMode::Request,
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    env::set_var("RUST_BACKTRACE", "1");
    let pool = db::create_connection_pool();
    println!("http://{}", env::var("HOST").expect("HOST is not set"));

    HttpServer::new(move || {
        let mut cors = Cors::default();
        let cors_str = env::var("CORS").unwrap_or_default();
        if cors_str.len() > 0 {
            let cors_split = cors_str.split("|");
            for domain in cors_split {
                cors = cors.allowed_origin(&domain);
            }
        } else {
            cors = cors.allow_any_origin();
        }
        cors = cors.allow_any_method().allow_any_header();

        App::new()
            .wrap(Logger::default())
            .wrap(sentry_actix::Sentry::new())
            .wrap(cors)
            .wrap(DefaultHeaders::new().header("Access-Control-Allow-Credentials", "true"))
            .data(pool.clone())
            .app_data(actix_web_validator::JsonConfig::default().limit(1024 * 1024 * 1))
            .service(web::scope("/v1").service(routes::scope()))
            .service(routes::scope())
    })
    .bind(env::var("HOST").expect("HOST is not set"))?
    .run()
    .await
}
