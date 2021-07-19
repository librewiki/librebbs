use crate::models::Board;
use crate::schema::topics;
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use std::net::IpAddr;

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
    pub fn get_all(conn: &MysqlConnection, limit: i32, offset: i32) -> Result<Vec<Self>> {
        let posts = topics::table
            .order_by(topics::id.desc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Self>(conn)?;
        Ok(posts)
    }
    pub fn find_by_id(conn: &MysqlConnection, id: i32) -> Result<Option<Self>> {
        let post = topics::table.find(id).first::<Self>(conn).optional()?;
        Ok(post)
    }

    pub fn has_ipv6(&self) -> bool {
        self.author_ip[4..].iter().any(|x| *x != 0u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_connection;

    #[test]
    fn test_topic() {
        use std::convert::TryInto;
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
