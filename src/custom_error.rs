use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub struct CustomError {
    err: anyhow::Error,
}

impl error::ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body("INTERNAL SERVER ERROR")
    }
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<r2d2::Error> for CustomError {
    fn from(err: r2d2::Error) -> CustomError {
        CustomError {
            err: anyhow::anyhow!(err),
        }
    }
}

impl From<actix_web::error::BlockingError<anyhow::Error>> for CustomError {
    fn from(err: actix_web::error::BlockingError<anyhow::Error>) -> CustomError {
        CustomError {
            err: anyhow::anyhow!(err),
        }
    }
}

impl From<anyhow::Error> for CustomError {
    fn from(err: anyhow::Error) -> CustomError {
        CustomError { err }
    }
}
