use crate::models::{Board, Comment};
use crate::schema::{comments, topics};
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use std::{
    convert::TryInto,
    hash::Hash,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
pub struct Topic {
    pub id: i32,
    pub board_id: i32,
    pub title: String,
    pub author_id: Option<i32>,
    pub author_name: Option<String>,
    pub author_ip: Vec<u8>,
    pub is_closed: bool,
    pub is_suspended: bool,
    pub is_hidden: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Identifiable, AsChangeset, Debug)]
#[table_name = "topics"]
pub struct TopicForm {
    pub id: i32,
    pub is_closed: Option<bool>,
    pub is_suspended: Option<bool>,
    pub is_hidden: Option<bool>,
}

impl TopicForm {
    pub fn save(&self, conn: &MysqlConnection) -> Result<Topic> {
        let topic = self.save_changes::<Topic>(conn)?;
        Ok(topic)
    }
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct TopicPublic {
    pub id: i32,
    pub board_id: i32,
    pub title: String,
    pub author_id: Option<i32>,
    pub author_name: String,
    pub is_closed: bool,
    pub is_suspended: bool,
    pub is_hidden: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "topics"]
struct NewTopic<'a> {
    pub board_id: i32,
    pub title: &'a str,
    pub author_id: Option<i32>,
    pub author_name: Option<&'a str>,
    pub author_ip: Vec<u8>,
}

impl Topic {
    pub fn create(
        conn: &MysqlConnection,
        board: &Board,
        title: &str,
        author_id: Option<i32>,
        author_name: Option<&str>,
        author_ip: &IpAddr,
    ) -> Result<()> {
        let ip_bin: Vec<u8> = match &author_ip {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let new_topic = NewTopic {
            board_id: board.id,
            author_id,
            author_name,
            author_ip: ip_bin,
            title,
        };
        diesel::insert_into(topics::table)
            .values(new_topic)
            .execute(conn)?;
        Ok(())
    }

    pub fn get_latest(conn: &MysqlConnection) -> Result<Self> {
        let topics = topics::table.order_by(topics::id.desc()).first(conn)?;
        Ok(topics)
    }

    pub fn get_all(conn: &MysqlConnection, limit: i32, offset: i32) -> Result<Vec<Self>> {
        let topics = topics::table
            .order_by(topics::id.desc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Self>(conn)?;
        Ok(topics)
    }

    pub fn find_by_id(conn: &MysqlConnection, id: i32) -> Result<Self> {
        let post = topics::table.find(id).first::<Self>(conn)?;
        Ok(post)
    }

    pub fn get_comments(
        &self,
        conn: &MysqlConnection,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Comment>> {
        let comments = comments::table
            .filter(comments::topic_id.eq(self.id))
            .order_by(comments::id.asc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Comment>(conn)?;
        Ok(comments)
    }

    pub fn has_ipv6(&self) -> bool {
        self.author_ip[4..].iter().any(|x| *x != 0u8)
    }

    pub fn get_public(&self) -> TopicPublic {
        TopicPublic {
            id: self.id,
            board_id: self.board_id,
            title: self.title.clone(),
            author_id: self.author_id,
            author_name: if let Some(name) = &self.author_name {
                name.clone()
            } else {
                self.get_ip_string()
            },
            is_closed: self.is_closed,
            is_suspended: self.is_suspended,
            is_hidden: self.is_hidden,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// Make updated_at recent
    pub fn touch(self, conn: &MysqlConnection) -> Result<()> {
        diesel::update(topics::table)
            .filter(topics::id.eq(self.id))
            .set(topics::updated_at.eq(diesel::dsl::now))
            .execute(conn)?;
        Ok(())
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
            assert_eq!("test title", topics[0].title);
            assert_eq!(Some("test_author".to_owned()), topics[0].author_name);
            assert_eq!(true, topics[0].has_ipv6());
            let arr: &[u8; 16] = topics[0].author_ip[..].try_into().expect("must succeed");
            assert_eq!(ip, IpAddr::from(*arr));

            let ip = IpAddr::from_str("127.0.0.3").expect("must succeed");
            Topic::create(&conn, &boards[0], "test title 2", None, None, &ip)
                .expect("must succeed");
            let topics = Topic::get_all(&conn, 1, 0).expect("must succeed");
            assert_eq!("test title 2", topics[0].title);
            assert_eq!(None, topics[0].author_name);
            assert_eq!(false, topics[0].has_ipv6());
            let arr: &[u8; 4] = topics[0].author_ip[0..4].try_into().expect("must succeed");
            assert_eq!(ip, IpAddr::from(*arr));
            assert_eq!(false, topics[0].is_closed);
            assert_eq!(false, topics[0].is_suspended);
            assert_eq!(false, topics[0].is_hidden);
            Ok(())
        });
    }
}
