use chrono::{NaiveDateTime, Utc};

pub struct Tag {
    pub(crate) id: String,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub color: String,
}

impl Tag {
    pub fn new(name: String, description: Option<String>, color: String, username: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            username,
            description,
            created_at: Utc::now().naive_utc(),
            color,
        }
    }
}
