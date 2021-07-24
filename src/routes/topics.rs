use crate::auth::{Profile, UserInfo};
use crate::connection_info::ConnectionInfo;
use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::{Board, Comment, CommentPublic, Topic};
use actix_web::{
    error::BlockingError,
    get, post, web,
    web::{block, Data, Json, Path, Query},
    HttpResponse, Scope,
};
use derive_more::Display;
use diesel::Connection;

#[get("{topic_id}")]
async fn get_topic(
    pool: Data<DbPool>,
    Path((topic_id,)): Path<(i32,)>,
) -> Result<HttpResponse, CustomError> {
    let conn = pool.get()?;
    match block(move || Topic::find_by_id(&conn, topic_id)).await {
        Ok(topic) => Ok(HttpResponse::Ok()
            .set_header("Cache-Control", "max-age=86400")
            .json(topic.get_public())),
        Err(BlockingError::Error(_)) => Ok(HttpResponse::NotFound().body("Topic is not found")),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

#[derive(Deserialize, Debug)]
struct GetCommentsQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[get("{topic_id}/comments")]
async fn get_topic_comments(
    pool: Data<DbPool>,
    Path((topic_id,)): Path<(i32,)>,
    query: Query<GetCommentsQuery>,
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
                .get_comments(&conn, limit, offset)
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
                .map(|x| x.get_public(false))
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

#[derive(Deserialize, Debug)]
struct PostTopicRequest {
    board_id: i32,
    title: String,
    content: String,
}

#[post("")]
async fn post_topic(
    ConnectionInfo { ip }: ConnectionInfo,
    Json(PostTopicRequest {
        board_id,
        title,
        content,
    }): Json<PostTopicRequest>,
    user_info: Option<UserInfo>,
    pool: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        BoardNotFound,
        OtherError(anyhow::Error),
    }

    let conn = pool.get()?;
    let profile = match user_info {
        Some(UserInfo { token, .. }) => Some(Profile::get(&token).await?),
        None => None,
    };

    let res = block::<_, Topic, ErrorKind>(move || {
        let board = Board::find_by_id(&conn, board_id).map_err(|_| ErrorKind::BoardNotFound)?;
        let topic = conn
            .transaction::<Topic, _, _>(|| match profile {
                Some(Profile { id, username, .. }) => {
                    Topic::create(&conn, &board, &title, Some(id), Some(&username), &ip)?;
                    let topic = Topic::get_latest(&conn)?;
                    Comment::create(&conn, &topic, &content, Some(id), Some(&username), &ip)?;
                    Ok(topic)
                }
                None => {
                    Topic::create(&conn, &board, &title, None, None, &ip)?;
                    let topic = Topic::get_latest(&conn)?;
                    Comment::create(&conn, &topic, &content, None, None, &ip)?;
                    Ok(topic)
                }
            })
            .map_err(|e| ErrorKind::OtherError(e))?;
        Ok(topic)
    })
    .await;
    match res {
        Ok(topic) => Ok(HttpResponse::Ok().json(topic.get_public())),
        Err(BlockingError::Error(ErrorKind::BoardNotFound)) => {
            Ok(HttpResponse::NotFound().body("Board is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

#[derive(Deserialize, Debug)]
struct PostCommentRequest {
    content: String,
}

#[post("{topic_id}/comments")]
async fn post_topic_comments(
    ConnectionInfo { ip }: ConnectionInfo,
    Json(PostCommentRequest { content }): Json<PostCommentRequest>,
    Path((topic_id,)): Path<(i32,)>,
    user_info: Option<UserInfo>,
    pool: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        TopicNotFound,
        OtherError(anyhow::Error),
    }

    let conn = pool.get()?;
    let profile = match user_info {
        Some(UserInfo { token, .. }) => Some(Profile::get(&token).await?),
        None => None,
    };

    let res = block::<_, (), ErrorKind>(move || {
        let topic = Topic::find_by_id(&conn, topic_id).map_err(|_| ErrorKind::TopicNotFound)?;
        conn.transaction::<(), _, _>(|| {
            match profile {
                Some(Profile { id, username, .. }) => {
                    Comment::create(&conn, &topic, &content, Some(id), Some(&username), &ip)?;
                }
                None => {
                    Comment::create(&conn, &topic, &content, None, None, &ip)?;
                }
            };
            Ok(())
        })
        .map_err(|e| ErrorKind::OtherError(e))?;
        Ok(())
    })
    .await;
    match res {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(BlockingError::Error(ErrorKind::TopicNotFound)) => {
            Ok(HttpResponse::NotFound().body("Topic is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

pub fn scope() -> Scope {
    web::scope("/topics")
        .service(post_topic)
        .service(get_topic)
        .service(get_topic_comments)
        .service(post_topic_comments)
}
