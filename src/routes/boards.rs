use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::Board;
use actix_web::{
    get, web,
    web::{block, Data},
    HttpResponse, Scope,
};
use jsonapi::api::*;
use jsonapi::jsonapi_model;
use jsonapi::model::*;

jsonapi_model!(Board; "board");

#[get("")]
async fn get(pool: Data<DbPool>) -> Result<HttpResponse, CustomError> {
    let conn = pool.get()?;
    let boards = block(move || Board::get_all(&conn)).await?;
    let doc = vec_to_jsonapi_document(boards);
    Ok(HttpResponse::Ok().json(doc))
}

pub fn scope() -> Scope {
    web::scope("/boards").service(get)
}
