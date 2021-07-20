use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::{CommentPublic, Topic};
use actix_web::error::BlockingError;
use actix_web::{
    get, web,
    web::{block, Data, Path, Query},
    HttpResponse, Scope,
};
use derive_more::Display;

#[get("{topic_id}")]
async fn get_topic(
    pool: Data<DbPool>,
    Path((topic_id,)): Path<(i32,)>,
) -> Result<HttpResponse, CustomError> {
    let conn = pool.get()?;
    match block(move || Topic::find_by_id(&conn, topic_id)).await {
        Ok(topic) => Ok(HttpResponse::Ok().json(topic.get_public())),
        Err(BlockingError::Error(_)) => Ok(HttpResponse::NotFound().body("Topic is not found")),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

#[derive(Deserialize, Debug)]
struct GetCommentsRequestQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[get("{topic_id}/comments")]
async fn get_topic_comments(
    pool: Data<DbPool>,
    Path((topic_id,)): Path<(i32,)>,
    query: Query<GetCommentsRequestQuery>,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        TopicNotFound,
        OtherError(anyhow::Error),
    }

    let limit = query.limit.unwrap_or(10);
    let limit = if limit > 20 { 20 } else { limit };
    let offset = query.offset.unwrap_or(0);

    let conn = pool.get()?;
    let res = block(move || {
        if let Ok(topic) = Topic::find_by_id(&conn, topic_id) {
            let comments = topic
                .get_comments(&conn, limit, offset, false)
                .map_err(|e| ErrorKind::OtherError(e))?;
            Ok(comments)
        } else {
            Err(ErrorKind::TopicNotFound)
        }
    })
    .await;
    match res {
        Ok(comments) => {
            let comments = comments
                .iter()
                .map(|x| x.get_public())
                .collect::<Vec<CommentPublic>>();
            Ok(HttpResponse::Ok().json(comments))
        }
        Err(BlockingError::Error(ErrorKind::TopicNotFound)) => {
            Ok(HttpResponse::NotFound().body("Topic is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

pub fn scope() -> Scope {
    web::scope("/topics")
        .service(get_topic)
        .service(get_topic_comments)
}
