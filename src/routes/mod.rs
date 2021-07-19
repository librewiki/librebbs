pub mod auth;
pub mod boards;
pub mod me;

use actix_web::{get, web, Error, HttpResponse, Scope};

#[get("/")]
async fn get() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hi!"))
}

pub fn scope() -> Scope {
    web::scope("")
        .service(get)
        .service(boards::scope())
        .service(auth::scope())
        .service(me::scope())
}
