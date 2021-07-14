pub mod boards;

use actix_web::{get, Error, HttpResponse, web, Scope};

#[get("/")]
async fn get() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hi!"))
}

pub fn scope() -> Scope {
    web::scope("")
        .service(get)
        .service(boards::scope())
}
