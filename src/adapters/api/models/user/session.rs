use chrono::NaiveDateTime;

pub struct SessionDto {
    pub username: String,
    pub session_id: String,
    pub expires: NaiveDateTime,
}