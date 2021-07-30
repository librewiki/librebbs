use std::net::IpAddr;

use crate::auth::{Profile, UserInfo};
use crate::connection_info::ConnectionInfo;
use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::{Comment, CommentForm, Log, LogContent, LogType};
use actix_web::error::BlockingError;
use actix_web::{
    get, put, web,
    web::{block, Data, Path, Query},
    HttpResponse, Scope,
};
use actix_web_validator::Json;
use derive_more::Display;
use diesel::{Connection, MysqlConnection};
use validator::Validate;

#[derive(Deserialize, Debug)]
struct GetCommentQuery {
    show_hidden: Option<bool>,
}

#[get("{comment_id}")]
async fn get_comment(
    pool: Data<DbPool>,
    UserInfo { token, .. }: UserInfo,
    Path((comment_id,)): Path<(i32,)>,
    query: Query<GetCommentQuery>,
) -> Result<HttpResponse, CustomError> {
    let show_hidden = query.show_hidden.unwrap_or(false);
    if show_hidden {
        let profile = match token {
            Some(token) => Profile::get(&token).await?,
            None => return Ok(HttpResponse::Unauthorized().body("TokenMissing")),
        };
        if !profile.is_admin() {
            return Ok(HttpResponse::Forbidden().finish());
        }
    }
    let conn = pool.get()?;
    match block(move || Comment::find_by_id(&conn, comment_id)).await {
        Ok(comment) => Ok(HttpResponse::Ok().json(comment.get_public(show_hidden))),
        Err(BlockingError::Error(_)) => Ok(HttpResponse::NotFound().body("Topic is not found")),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

#[derive(Deserialize, Validate, Debug)]
struct PutCommentStatusRequest {
    is_hidden: Option<bool>,
}

fn log_put_comment_status(
    conn: &MysqlConnection,
    comment_id: i32,
    user_id: Option<i32>,
    user_name: Option<&str>,
    user_ip: &IpAddr,
    req_status: PutCommentStatusRequest,
) -> anyhow::Result<()> {
    let log_type = match req_status {
        PutCommentStatusRequest {
            is_hidden: Some(true),
            ..
        } => LogType::HideComment,
        PutCommentStatusRequest {
            is_hidden: Some(false),
            ..
        } => LogType::UnhideComment,
        _ => return Ok(()),
    };
    Log::add(
        &conn,
        &log_type,
        &LogContent { target: comment_id },
        user_id,
        user_name,
        &user_ip,
    )?;
    Ok(())
}

#[put("{comment_id}/status")]
async fn put_comment_status(
    pool: Data<DbPool>,
    UserInfo { token, .. }: UserInfo,
    Path((comment_id,)): Path<(i32,)>,
    Json(req_status): Json<PutCommentStatusRequest>,
    ConnectionInfo { ip }: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        CommentNotFound,
        OtherError(anyhow::Error),
    }
    impl From<diesel::result::Error> for ErrorKind {
        fn from(error: diesel::result::Error) -> Self {
            ErrorKind::OtherError(error.into())
        }
    }

    let profile = match token {
        Some(token) => Profile::get(&token).await?,
        None => return Ok(HttpResponse::Unauthorized().body("TokenMissing")),
    };

    if !profile.is_admin() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let conn = pool.get()?;

    let res = block(move || {
        conn.transaction::<Comment, _, _>(|| {
            Comment::find_by_id(&conn, comment_id).map_err(|_| ErrorKind::CommentNotFound)?;
            let comment_changes = CommentForm {
                id: comment_id,
                is_hidden: req_status.is_hidden,
            };
            let changed = comment_changes
                .save(&conn)
                .map_err(|e| ErrorKind::OtherError(e))?;
            log_put_comment_status(
                &conn,
                comment_id,
                Some(profile.id),
                Some(&profile.username),
                &ip,
                req_status,
            )
            .map_err(|e| ErrorKind::OtherError(e))?;
            Ok(changed)
        })
    })
    .await;

    match res {
        Ok(comment) => Ok(HttpResponse::Ok().json(comment.get_public(true))),
        Err(BlockingError::Error(ErrorKind::CommentNotFound)) => {
            Ok(HttpResponse::NotFound().body("Comment is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

pub fn scope() -> Scope {
    web::scope("/comments")
        .service(get_comment)
        .service(put_comment_status)
}
