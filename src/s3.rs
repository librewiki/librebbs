use anyhow::Result;
use nanoid::nanoid;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use std::{env, str::FromStr};

pub async fn upload(bytes: &[u8], ext: &str) -> Result<String> {
    let s3_region = env::var("S3_REGION").expect("S3_REGION is not set");
    let s3_region = Region::from_str(&s3_region).expect("S3_REGION is invalid");
    let s3_bucket = env::var("S3_BUCKET").expect("S3_BUCKET is not set");
    let filename = nanoid!();
    let client = S3Client::new(s3_region);
    let mime = mime_guess::from_ext(ext).first_or_octet_stream();
    let path = format!("bbs/{}.{}", filename, ext);

    client
        .put_object(PutObjectRequest {
            body: Some(bytes.to_vec().into()),
            bucket: s3_bucket,
            key: path.clone(),
            content_type: Some(mime.to_string()),
            content_md5: None,
            ..Default::default()
        })
        .await?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_upload() {
        dotenv().ok();
        upload(include_bytes!("../LICENSE"), "txt")
            .await
            .expect("must succeed");
    }
}
