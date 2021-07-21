use crate::auth::UserInfo;
use crate::custom_error::CustomError;
use actix_web::{client::Client, get, web, HttpResponse, Scope};
use anyhow::anyhow;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MwProfileResponse {
    sub: i64,
    username: String,
    editcount: i64,
    confirmed_email: bool,
    blocked: bool,
    registered: String,
    groups: Vec<String>,
    rights: Vec<String>,
    grants: Vec<String>,
    realname: String,
    email: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Profile {
    id: i64,
    username: String,
    confirmed_email: bool,
    blocked: bool,
    groups: Vec<String>,
    rights: Vec<String>,
    email: String,
}

#[get("")]
async fn get_me(UserInfo { token, .. }: UserInfo) -> Result<HttpResponse, CustomError> {
    let client = Client::default();
    let mut res = client
        .get("https://librewiki.net/rest.php/oauth2/resource/profile")
        .set_header("Accept", "application/json")
        .set_header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| anyhow!(format!("{}", e)))?;
    // MW gives incorrect content-type header
    let body = res.body().await.map_err(|e| anyhow!(format!("{}", e)))?;
    let data: MwProfileResponse =
        serde_json::from_slice(&body).map_err(|e| anyhow!(format!("{}", e)))?;
    let resp = Profile {
        id: data.sub,
        username: data.username,
        confirmed_email: data.confirmed_email,
        blocked: data.blocked,
        groups: data.groups,
        rights: data.rights,
        email: data.email,
    };

    Ok(HttpResponse::Ok()
        .set_header("Cache-Control", "private, max-age=600")
        .json(resp))
}

pub fn scope() -> Scope {
    web::scope("/me").service(get_me)
}
