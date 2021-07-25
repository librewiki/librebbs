use crate::auth::RefreshToken;
use crate::custom_error::CustomError;
use actix_web::{
    client::Client,
    http::{Cookie, StatusCode},
    post, web, HttpResponse, Scope,
};
use actix_web_validator::Json;
use anyhow::anyhow;
use std::env;
use time::Duration;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginRequest {
    code: String,
}

#[derive(Serialize, Debug)]
struct OauthAccessTokenCodeRequest {
    grant_type: String,
    code: String,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Debug)]
struct OauthAccessTokenRefreshRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize, Debug)]
struct OauthAccessTokenResponse {
    access_token: String,
    refresh_token: String,
}

#[post("/login")]
async fn login(
    Json(LoginRequest { code }): Json<LoginRequest>,
) -> Result<HttpResponse, CustomError> {
    let client = Client::default();
    let mut res = client
        .post("https://librewiki.net/rest.php/oauth2/access_token")
        .send_form(&OauthAccessTokenCodeRequest {
            grant_type: "authorization_code".to_owned(),
            client_id: env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set"),
            client_secret: env::var("OAUTH_CLIENT_SECRET")
                .expect("OAUTH_CLIENT_SECRET must be set"),
            code,
        })
        .await
        .map_err(|e| anyhow!(format!("{}", e)))?;
    let data = res
        .json::<OauthAccessTokenResponse>()
        .limit(10 * 1024)
        .await
        .map_err(|e| anyhow!(format!("{}", e)))?;
    let cookie1 = Cookie::build("access_token", &data.access_token)
        .http_only(true)
        .max_age(Duration::days(28))
        .path("/")
        .finish();
    let cookie2 = Cookie::build("refresh_token", &data.refresh_token)
        .http_only(true)
        .max_age(Duration::days(28))
        .path("/")
        .finish();

    let mut response = HttpResponse::NoContent().finish();
    response
        .add_cookie(&cookie1)
        .map_err(|e| anyhow!(format!("{}", e)))?;
    response
        .add_cookie(&cookie2)
        .map_err(|e| anyhow!(format!("{}", e)))?;

    Ok(response)
}

#[post("/logout")]
async fn logout() -> Result<HttpResponse, CustomError> {
    let cookie1 = Cookie::build("access_token", "")
        .http_only(true)
        .max_age(Duration::zero())
        .path("/")
        .finish();
    let cookie2 = Cookie::build("refresh_token", "")
        .http_only(true)
        .max_age(Duration::zero())
        .path("/")
        .finish();

    let mut response = HttpResponse::NoContent().finish();
    response
        .add_cookie(&cookie1)
        .map_err(|e| anyhow!(format!("{}", e)))?;
    response
        .add_cookie(&cookie2)
        .map_err(|e| anyhow!(format!("{}", e)))?;

    Ok(response)
}

#[post("/refresh")]
async fn refresh(
    RefreshToken { refresh_token }: RefreshToken,
) -> Result<HttpResponse, CustomError> {
    let client = Client::default();
    let mut res = client
        .post("https://librewiki.net/rest.php/oauth2/access_token")
        .send_form(&OauthAccessTokenRefreshRequest {
            grant_type: "refresh_token".to_owned(),
            client_id: env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set"),
            client_secret: env::var("OAUTH_CLIENT_SECRET")
                .expect("OAUTH_CLIENT_SECRET must be set"),
            refresh_token,
        })
        .await
        .map_err(|e| anyhow!(format!("{}", e)))?;
    match res.status() {
        StatusCode::OK => {}
        _ => {
            return Ok(HttpResponse::Unauthorized().finish());
        }
    }
    let data = res
        .json::<OauthAccessTokenResponse>()
        .limit(10 * 1024)
        .await
        .map_err(|e| anyhow!(format!("{}", e)))?;

    let cookie1 = Cookie::build("access_token", &data.access_token)
        .http_only(true)
        .max_age(Duration::days(28))
        .path("/")
        .finish();
    let cookie2 = Cookie::build("refresh_token", &data.refresh_token)
        .http_only(true)
        .max_age(Duration::days(28))
        .path("/")
        .finish();

    let mut response = HttpResponse::NoContent().finish();
    response
        .add_cookie(&cookie1)
        .map_err(|e| anyhow!(format!("{}", e)))?;
    response
        .add_cookie(&cookie2)
        .map_err(|e| anyhow!(format!("{}", e)))?;

    Ok(response)
}

pub fn scope() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(logout)
        .service(refresh)
}
