use crate::db::DbPool;
use crate::models::Board;
use actix_web::{
    get, web,
    web::{block, Data, Query},
    Error, HttpResponse, Scope
};
use jsonapi::api::*;
use jsonapi::jsonapi_model;
use jsonapi::model::*;

jsonapi_model!(Board; "marker");

#[get("/")]
async fn get(pool: Data<DbPool>) -> Result<HttpResponse, Error> {
    let conn = pool.get().map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    let boards = block(move || Board::get_all(&conn)).await.map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    let doc = vec_to_jsonapi_document(boards);
    Ok(HttpResponse::Ok().json(doc))
}

pub fn scope() -> Scope {
    web::scope("/boards")
        .service(get)
}
