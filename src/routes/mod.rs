mod auth;
mod boards;
mod files;
mod me;
mod topics;

use actix_web::{get, web, Error, HttpResponse, Scope};

#[get("/")]
async fn get() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hi!"))
}

pub fn scope() -> Scope {
    web::scope("")
        .service(get)
        .service(auth::scope())
        .service(me::scope())
        .service(files::scope())
        .service(boards::scope())
        .service(topics::scope())
}
