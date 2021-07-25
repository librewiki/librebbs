mod board;
mod comment;
mod topic;
pub use board::Board;
pub use comment::{Comment, CommentForm, CommentPublic};
pub use topic::{Topic, TopicForm, TopicPublic};

use actix_web::{
    http::header::{ETag, EntityTag, IF_NONE_MATCH},
    HttpRequest, HttpResponse,
};
use serde::ser::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn etag_equals(request: &HttpRequest, etag: &str) -> bool {
    if let Some(v) = request.headers().get(IF_NONE_MATCH) {
        let request_etag = v.to_str().unwrap_or_default().replace("\"", "");
        return request_etag == etag;
    }
    return false;
}

pub trait PublicEntity {
    fn get_etag(&self) -> String;
    fn cache_response(&self, request: &HttpRequest) -> HttpResponse;
}

impl<T> PublicEntity for T
where
    T: Hash + Serialize,
{
    fn get_etag(&self) -> String {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        let hash = s.finish();
        hash.to_string()
    }
    fn cache_response(&self, request: &HttpRequest) -> HttpResponse {
        let etag = self.get_etag();
        if etag_equals(&request, &etag) {
            HttpResponse::NotModified().finish()
        } else {
            HttpResponse::Ok()
                .set(ETag(EntityTag::strong(etag)))
                .json(&self)
        }
    }
}
