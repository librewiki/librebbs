use crate::{custom_error::CustomError, s3};
use actix_web::{post, web, web::Json, HttpResponse, Scope};
use anyhow::anyhow;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct PostFileRequest {
    filename: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct PostFileResponse {
    path: String,
}

#[post("")]
async fn post_files(
    Json(PostFileRequest { filename, content }): Json<PostFileRequest>,
) -> Result<HttpResponse, CustomError> {
    let bytes = base64::decode(content).map_err(|e| anyhow!(format!("{}", e)))?;
    let ext = Path::new(&filename)
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let path = s3::upload(&bytes, ext).await?;
    Ok(HttpResponse::Ok().json(PostFileResponse { path }))
}

pub fn scope() -> Scope {
    let json_cfg = web::JsonConfig::default().limit(1024 * 1024 * 15); // 15 MiB (base64 - about 30% larger than original)
    web::scope("/files").app_data(json_cfg).service(post_files)
}
