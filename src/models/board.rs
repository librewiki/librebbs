use crate::models::Topic;
use crate::schema::{boards, topics};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
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
        include_hidden: bool,
    ) -> Result<Vec<Topic>> {
        let mut query = topics::table.into_boxed();
        query = query.filter(topics::board_id.eq(self.id));
        if !include_hidden {
            query = query.filter(topics::is_hidden.eq(false));
        }
        let topics = query
            .order_by(topics::is_pinned.desc())
            .then_order_by(topics::updated_at.desc())
            .limit(limit.into())
            .offset(offset.into())
            .load::<Topic>(conn)?;
        Ok(topics)
    }
    pub fn get_public(&self) -> BoardPublic {
        BoardPublic {
            id: self.id,
            display_name: self.display_name.clone(),
            name: self.name.clone(),
            is_active: self.is_active,
            created_at: DateTime::<Utc>::from_utc(self.created_at, Utc),
            updated_at: DateTime::<Utc>::from_utc(self.updated_at, Utc),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct BoardPublic {
    pub id: i32,
    pub display_name: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
