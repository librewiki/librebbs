use std::net::IpAddr;

use crate::auth::{Profile, UserInfo};
use crate::connection_info::ConnectionInfo;
use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::{
    Board, Comment, CommentPublic, Log, LogContent, LogType, PublicEntity, Topic, TopicForm,
};
use actix_web::{
    error::BlockingError,
    get, post, put, web,
    web::{block, Data, Path, Query},
    HttpRequest, HttpResponse, Scope,
};
use actix_web_validator::Json;
use derive_more::Display;
use diesel::{Connection, MysqlConnection};
use validator::Validate;

#[get("{topic_id}")]
async fn get_topic(
    pool: Data<DbPool>,
    Path((topic_id,)): Path<(i32,)>,
    request: HttpRequest,
) -> Result<HttpResponse, CustomError> {
    let conn = pool.get()?;
    match block(move || Topic::find_by_id(&conn, topic_id)).await {
        Ok(topic) => {
            if topic.is_hidden {
                Ok(HttpResponse::Forbidden().body("Topic is hidden"))
            } else {
                Ok(topic.get_public().cache_response(&request))
            }
        }
        Err(BlockingError::Error(_)) => Ok(HttpResponse::NotFound().body("Topic is not found")),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

#[derive(Deserialize, Validate, Debug)]
struct PutTopicStatusRequest {
    is_closed: Option<bool>,
    is_suspended: Option<bool>,
    is_hidden: Option<bool>,
}

fn log_put_topic_status(
    conn: &MysqlConnection,
    topic_id: i32,
    user_id: Option<i32>,
    user_name: Option<&str>,
    user_ip: &IpAddr,
    req_status: PutTopicStatusRequest,
) -> anyhow::Result<()> {
    let log_type = match req_status {
        PutTopicStatusRequest {
            is_closed: Some(true),
            ..
        } => LogType::CloseTopic,
        PutTopicStatusRequest {
            is_closed: Some(false),
            ..
        } => LogType::UncloseTopic,
        PutTopicStatusRequest {
            is_hidden: Some(true),
            ..
        } => LogType::HideTopic,
        PutTopicStatusRequest {
            is_hidden: Some(false),
            ..
        } => LogType::UnhideTopic,
        PutTopicStatusRequest {
            is_suspended: Some(true),
            ..
        } => LogType::SuspendTopic,
        PutTopicStatusRequest {
            is_suspended: Some(false),
            ..
        } => LogType::UnsuspendTopic,
        _ => return Ok(()),
    };
    Log::add(
        &conn,
        &log_type,
        &LogContent { target: topic_id },
        user_id,
        user_name,
        &user_ip,
    )?;
    Ok(())
}

#[put("{topic_id}/status")]
async fn put_topic_status(
    pool: Data<DbPool>,
    UserInfo { token, .. }: UserInfo,
    Path((topic_id,)): Path<(i32,)>,
    Json(req_status): Json<PutTopicStatusRequest>,
    ConnectionInfo { ip }: ConnectionInfo,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        TopicNotFound,
        OtherError(anyhow::Error),
    }
    impl From<diesel::result::Error> for ErrorKind {
        fn from(error: diesel::result::Error) -> Self {
            ErrorKind::OtherError(error.into())
        }
    }

    let profile = Profile::get(&token).await?;

    if !profile.is_admin() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let conn = pool.get()?;
    let res = block(move || {
        conn.transaction::<Topic, _, _>(|| {
            Topic::find_by_id(&conn, topic_id).map_err(|_| ErrorKind::TopicNotFound)?;
            let topic_changes = TopicForm {
                id: topic_id,
                is_closed: req_status.is_closed,
                is_suspended: req_status.is_suspended,
                is_hidden: req_status.is_hidden,
            };
            let changed = topic_changes
                .save(&conn)
                .map_err(|e| ErrorKind::OtherError(e))?;
            log_put_topic_status(
                &conn,
                topic_id,
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
        Ok(topic) => Ok(HttpResponse::Ok().json(topic.get_public())),
        Err(BlockingError::Error(ErrorKind::TopicNotFound)) => {
            Ok(HttpResponse::NotFound().body("Topic is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
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
    request: HttpRequest,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        TopicNotFound,
        TopicIsHidden,
        OtherError(anyhow::Error),
    }

    let limit = query.limit.unwrap_or(10);
    let limit = if limit > 20 { 20 } else { limit };
    let offset = query.offset.unwrap_or(0);

    let conn = pool.get()?;
    let res = block(move || {
        if let Ok(topic) = Topic::find_by_id(&conn, topic_id) {
            if topic.is_hidden {
                return Err(ErrorKind::TopicIsHidden);
            }
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
            Ok(comments.cache_response(&request))
        }
        Err(BlockingError::Error(ErrorKind::TopicNotFound)) => {
            Ok(HttpResponse::NotFound().body("Topic is not found"))
        }
        Err(BlockingError::Error(ErrorKind::TopicIsHidden)) => {
            Ok(HttpResponse::Forbidden().body("Topic is hidden"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

#[derive(Deserialize, Validate, Debug)]
struct PostTopicRequest {
    board_id: i32,
    #[validate(length(min = 1, max = 100))]
    title: String,
    #[validate(length(min = 1, max = 100000))]
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

#[derive(Deserialize, Validate, Debug)]
struct PostCommentRequest {
    #[validate(length(min = 1, max = 100000))]
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
        TopicIsHidden,
        TopicIsSuspended,
        TopicIsClosed,
        OtherError(anyhow::Error),
    }

    let conn = pool.get()?;
    let profile = match user_info {
        Some(UserInfo { token, .. }) => Some(Profile::get(&token).await?),
        None => None,
    };

    let res = block::<_, (), ErrorKind>(move || {
        let topic = Topic::find_by_id(&conn, topic_id).map_err(|_| ErrorKind::TopicNotFound)?;
        if topic.is_hidden {
            return Err(ErrorKind::TopicIsHidden);
        } else if topic.is_closed {
            return Err(ErrorKind::TopicIsClosed);
        } else if topic.is_suspended {
            return Err(ErrorKind::TopicIsSuspended);
        }
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
        Err(BlockingError::Error(ErrorKind::TopicIsHidden)) => {
            Ok(HttpResponse::Forbidden().body("Topic is hidden"))
        }
        Err(BlockingError::Error(ErrorKind::TopicIsClosed)) => {
            Ok(HttpResponse::Forbidden().body("Topic is closed"))
        }
        Err(BlockingError::Error(ErrorKind::TopicIsSuspended)) => {
            Ok(HttpResponse::Forbidden().body("Topic is suspended"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

pub fn scope() -> Scope {
    web::scope("/topics")
        .service(post_topic)
        .service(get_topic)
        .service(put_topic_status)
        .service(get_topic_comments)
        .service(post_topic_comments)
}
