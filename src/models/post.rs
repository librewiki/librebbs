use crate::models::Board;
use crate::schema::posts;
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
pub struct Post {
    pub id: i32,
    pub board_id: i32,
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "posts"]
struct NewPost<'a> {
    pub board_id: i32,
    pub title: &'a str,
    pub content: &'a str,
}

impl Post {
    pub fn create(conn: &MysqlConnection, board: &Board, title: &str, content: &str) -> Result<()> {
        let new_post = NewPost {
            board_id: board.id,
            title,
            content,
        };
        diesel::insert_into(posts::table)
            .values(new_post)
            .execute(conn)?;
        Ok(())
    }
    pub fn get_all(conn: &MysqlConnection, limit: i32, offset: i32) -> Result<Vec<Self>> {
        let posts = posts::table
            .order_by(posts::id.desc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Self>(conn)?;
        Ok(posts)
    }
    pub fn find_by_id(conn: &MysqlConnection, id: i32) -> Result<Option<Self>> {
        let post = posts::table.find(id).first::<Self>(conn).optional()?;
        Ok(post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_connection;

    #[test]
    fn test_post() {
        let conn = create_connection();
        conn.test_transaction::<_, diesel::result::Error, _>(|| {
            let boards = Board::get_all(&conn).expect("A board must exist");
            Post::create(&conn, &boards[0], "test title", "test content").expect("must succeed");
            Post::create(&conn, &boards[0], "test title 2", "test content 2")
                .expect("must succeed");
            let post = Post::get_all(&conn, 1, 0).expect("must succeed");
            assert_eq!("test title 2", post[0].title);
            assert_eq!("test content 2", post[0].content);

            Ok(())
        });
    }
}
