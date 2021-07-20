use crate::models::Topic;
use crate::schema::{boards, topics};
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
pub struct Board {
    pub id: i32,
    pub display_name: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Board {
    pub fn get_all(conn: &MysqlConnection) -> Result<Vec<Self>> {
        let results = boards::table.load::<Self>(conn)?;
        Ok(results)
    }
    pub fn find_by_id(conn: &MysqlConnection, id: i32) -> Result<Self> {
        let post = boards::table.find(id).first::<Self>(conn)?;
        Ok(post)
    }
    pub fn get_topics(
        &self,
        conn: &MysqlConnection,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Topic>> {
        let topics = topics::table
            .filter(topics::board_id.eq(self.id))
            .order_by(topics::id.desc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Topic>(conn)?;
        Ok(topics)
    }
}
