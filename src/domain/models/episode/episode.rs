
use chrono::{NaiveDateTime};

pub struct Episode {
    pub id: i32,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}