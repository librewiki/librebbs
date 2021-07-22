use crate::models::Topic;
use crate::schema::comments;
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
pub struct Comment {
    pub id: i32,
    pub topic_id: i32,
    pub content: String,
    pub author_id: Option<i32>,
    pub author_name: Option<String>,
    pub author_ip: Vec<u8>,
    pub is_hidden: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentPublic {
    pub id: i32,
    pub topic_id: i32,
    pub content: Option<String>,
    pub author_id: Option<i32>,
    pub author_name: String,
    pub is_hidden: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "comments"]
struct NewComment<'a> {
    pub topic_id: i32,
    pub content: &'a str,
    pub author_id: Option<i32>,
    pub author_name: Option<&'a str>,
    pub author_ip: Vec<u8>,
}

impl Comment {
    pub fn create(
        conn: &MysqlConnection,
        topic: &Topic,
        content: &str,
        author_id: Option<i32>,
        author_name: Option<&str>,
        author_ip: &IpAddr,
    ) -> Result<()> {
        let ip_bin: Vec<u8> = match &author_ip {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let new_comment = NewComment {
            topic_id: topic.id,
            content,
            author_id,
            author_name,
            author_ip: ip_bin,
        };
        diesel::insert_into(comments::table)
            .values(new_comment)
            .execute(conn)?;
        Ok(())
    }

    pub fn get_all(conn: &MysqlConnection, limit: i32, offset: i32) -> Result<Vec<Self>> {
        let comments = comments::table
            .order_by(comments::id.desc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Self>(conn)?;
        Ok(comments)
    }

    pub fn find_by_id(conn: &MysqlConnection, id: i32) -> Result<Option<Self>> {
        let post = comments::table.find(id).first::<Self>(conn).optional()?;
        Ok(post)
    }

    pub fn has_ipv6(&self) -> bool {
        self.author_ip[4..].iter().any(|x| *x != 0u8)
    }

    pub fn get_public(&self, show_hidden: bool) -> CommentPublic {
        CommentPublic {
            id: self.id,
            topic_id: self.topic_id,
            content: if show_hidden || !self.is_hidden {
                Some(self.content.clone())
            } else {
                None
            },
            author_id: self.author_id,
            author_name: if let Some(name) = &self.author_name {
                name.clone()
            } else {
                self.get_ip_string()
            },
            is_hidden: self.is_hidden,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    fn get_ip_string(&self) -> String {
        let x: &Vec<u8> = &self.author_ip;
        if x[4..].iter().any(|x| *x != 0u8) {
            // IPv6
            let arr: &[u8; 16] = x[..].try_into().unwrap();
            Ipv6Addr::from(*arr).to_string()
        } else {
            // IPv4
            let arr: &[u8; 4] = x[0..4].try_into().unwrap();
            Ipv4Addr::from(*arr).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_connection;
    use crate::models::Board;

    #[test]
    fn test_topic() {
        use std::str::FromStr;
        let conn = create_connection();
        conn.test_transaction::<_, diesel::result::Error, _>(|| {
            let boards = Board::get_all(&conn).expect("A board must exist");
            let ip = IpAddr::from_str("2001:db8::1").expect("must succeed");
            Topic::create(
                &conn,
                &boards[0],
                "test title",
                Some(3),
                Some("test_author"),
                &ip,
            )
            .expect("must succeed");
            let topics = Topic::get_all(&conn, 1, 0).expect("must succeed");
            Comment::create(
                &conn,
                &topics[0],
                "Test content",
                Some(3),
                Some("test_author"),
                &ip,
            )
            .expect("must succeed");
            let comments = Comment::get_all(&conn, 1, 0).expect("must succeed");
            assert_eq!("Test content", comments[0].content);
            assert_eq!(Some("test_author".to_owned()), comments[0].author_name);
            assert_eq!(true, comments[0].has_ipv6());
            let arr: &[u8; 16] = comments[0].author_ip[..].try_into().expect("must succeed");
            assert_eq!(ip, IpAddr::from(*arr));

            Ok(())
        });
    }
}
