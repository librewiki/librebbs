use crate::custom_error::CustomError;
use crate::db::DbPool;
use crate::models::{Board, BoardPublic, TopicPublic};
use actix_web::error::BlockingError;
use actix_web::{
    get, web,
    web::{block, Data, Path, Query},
    HttpResponse, Scope,
};
use derive_more::Display;

#[get("")]
async fn get_boards(pool: Data<DbPool>) -> Result<HttpResponse, CustomError> {
    let conn = pool.get()?;
    let boards = block(move || Board::get_all(&conn)).await?;
    let boards = boards
        .iter()
        .map(|x| x.get_public())
        .collect::<Vec<BoardPublic>>();

    Ok(HttpResponse::Ok()
        .set_header("Cache-Control", "max-age=86400")
        .json(boards))
}

#[derive(Deserialize, Debug)]
struct GetTopicsQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[get("{board_id}/topics")]
async fn get_board_topics(
    pool: Data<DbPool>,
    Path((board_id,)): Path<(i32,)>,
    query: Query<GetTopicsQuery>,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        BoardNotFound,
        OtherError(anyhow::Error),
    }

    let limit = query.limit.unwrap_or(10);
    let limit = if limit > 20 { 20 } else { limit };
    let offset = query.offset.unwrap_or(0);

    let conn = pool.get()?;
    let res = block(move || {
        let board = Board::find_by_id(&conn, board_id).map_err(|_| ErrorKind::BoardNotFound)?;
        let topics = board
            .get_topics(&conn, limit, offset, false)
            .map_err(|e| ErrorKind::OtherError(e))?;
        Ok(topics)
    })
    .await;
    match res {
        Ok(topics) => {
            let topics = topics
                .iter()
                .map(|x| x.get_public())
                .collect::<Vec<TopicPublic>>();
            Ok(HttpResponse::Ok().json(topics))
        }
        Err(BlockingError::Error(ErrorKind::BoardNotFound)) => {
            Ok(HttpResponse::NotFound().body("Board is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}



#[derive(Deserialize, Debug)]
struct GetSearchQuery {
    limit: Option<i32>,
    offset: Option<i32>,
    query: String,
}

#[get("{board_id}/search")]
async fn search_topics(
    pool: Data<DbPool>,
    Path((board_id,)): Path<(i32,)>,
    query: Query<GetSearchQuery>,
) -> Result<HttpResponse, CustomError> {
    #[derive(Debug, Display)]
    enum ErrorKind {
        BoardNotFound,
        OtherError(anyhow::Error),
    }

    let limit = query.limit.unwrap_or(10);
    let limit = if limit > 20 { 20 } else { limit };
    let offset = query.offset.unwrap_or(0);

    let conn = pool.get()?;
    let res = block(move || {
        let board = Board::find_by_id(&conn, board_id).map_err(|_| ErrorKind::BoardNotFound)?;
        let topics = board
            .search_topics(&conn, limit, offset, false, query.query.clone())
            .map_err(|e| ErrorKind::OtherError(e))?;
        Ok(topics)
    })
    .await;
    match res {
        Ok(topics) => {
            let topics = topics
                .iter()
                .map(|x| x.get_public())
                .collect::<Vec<TopicPublic>>();
            Ok(HttpResponse::Ok().json(topics))
        }
        Err(BlockingError::Error(ErrorKind::BoardNotFound)) => {
            Ok(HttpResponse::NotFound().body("Board is not found"))
        }
        Err(BlockingError::Error(ErrorKind::OtherError(e))) => Err(e.into()),
        Err(BlockingError::Canceled) => Err(BlockingError::Canceled.into()),
    }
}

pub fn scope() -> Scope {
    web::scope("/boards")
        .service(get_boards)
        .service(get_board_topics)
        .service(search_topics)
}
