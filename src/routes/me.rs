use crate::auth::{Profile, UserInfo};
use crate::custom_error::CustomError;
use actix_web::{get, web, HttpResponse, Scope};

#[get("")]
async fn get_me(UserInfo { token, .. }: UserInfo) -> Result<HttpResponse, CustomError> {
    if let Some(token) = token {
        let profile = Profile::get(&token).await?;
        Ok(HttpResponse::Ok()
            .set_header("Cache-Control", "private, max-age=86400")
            .set_header("Vary", "Cookie")
            .json(profile))
    } else {
        Ok(HttpResponse::Unauthorized().body("TokenMissing"))
    }
}

pub fn scope() -> Scope {
    web::scope("/me").service(get_me)
}
