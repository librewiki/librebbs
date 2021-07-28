use std::net::IpAddr;

use crate::schema::logs;
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum LogType {
    CloseTopic = 1,
    UncloseTopic = 2,
    HideTopic = 3,
    UnhideTopic = 4,
    SuspendTopic = 5,
    UnsuspendTopic = 6,
    HideComment = 7,
    UnhideComment = 8,
    PinTopic = 9,
    UnpinTopic = 10,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogContent {
    pub target: i32,
}

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
pub struct Log {
    pub id: i32,
    pub log_type_id: LogType,
    pub content: String,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub user_ip: Vec<u8>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "logs"]
struct NewLog<'a> {
    pub log_type_id: i32,
    pub content: &'a str,
    pub user_id: Option<i32>,
    pub user_name: Option<&'a str>,
    pub user_ip: Vec<u8>,
}

impl Log {
    pub fn add(
        conn: &MysqlConnection,
        log_type: &LogType,
        content: &LogContent,
        user_id: Option<i32>,
        user_name: Option<&str>,
        user_ip: &IpAddr,
    ) -> Result<()> {
        let ip_bin: Vec<u8> = match &user_ip {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let new_comment = NewLog {
            log_type_id: *log_type as i32,
            content: &serde_json::to_string(&content)?,
            user_id,
            user_name,
            user_ip: ip_bin,
        };
        diesel::insert_into(logs::table)
            .values(new_comment)
            .execute(conn)?;
        Ok(())
    }
}
