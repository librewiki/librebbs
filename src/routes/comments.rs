use crate::auth::{Profile, UserInfo};
use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::{Comment, CommentForm};
use actix_web::error::BlockingError;
use actix_web::{
    get, put, web,
    web::{block, Data, Path, Query},
    HttpResponse, Scope,
};
use actix_web_validator::Json;
use derive_more::Display;
use validator::Validate;

#[derive(Deserialize, Debug)]
struct GetCommentQuery {
    show_hidden: Option<bool>,
}

#[get("{comment_id}")]
async fn get_comment(
    pool: Data<DbPool>,
    user: Option<UserInfo>,
    Path((comment_id,)): Path<(i32,)>,
    query: Query<GetCommentQuery>,
) -> Result<HttpResponse, CustomError> {
    let show_hidden = query.show_hidden.unwrap_or(false);
    if show_hidden {
        match user {
            Some(UserInfo { token, .. }) => {
                if !Profile::get(&token).await?.is_admin() {
                    return Ok(HttpResponse::Forbidden().finish());
                }
            }
            None => {
                return Ok(HttpResponse::Forbidden().finish());
            }
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

#[put("{comment_id}/status")]
async fn put_comment_status(
    pool: Data<DbPool>,
    UserInfo { token, .. }: UserInfo,
    Path((comment_id,)): Path<(i32,)>,
    Json(req_status): Json<PutCommentStatusRequest>,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        CommentNotFound,
        OtherError(anyhow::Error),
    }

    if !Profile::get(&token).await?.is_admin() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let conn = pool.get()?;
    let res = block(move || {
        Comment::find_by_id(&conn, comment_id).map_err(|_| ErrorKind::CommentNotFound)?;
        let comment_changes = CommentForm {
            id: comment_id,
            is_hidden: req_status.is_hidden,
        };
        let changed = comment_changes
            .save(&conn)
            .map_err(|e| ErrorKind::OtherError(e))?;
        Ok(changed)
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
