use crate::auth::{Profile, UserInfo};
use crate::custom_error::CustomError;
use actix_web::{get, web, HttpResponse, Scope};

#[get("")]
async fn get_me(UserInfo { token, .. }: UserInfo) -> Result<HttpResponse, CustomError> {
    let profile = Profile::get(&token).await?;

    Ok(HttpResponse::Ok()
        .set_header("Cache-Control", "private, max-age=86400")
        .set_header("Vary", "Cookie")
        .json(profile))
}

pub fn scope() -> Scope {
    web::scope("/me").service(get_me)
}
