use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithoutPassword {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithAPiKey {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool,
    pub api_key: Option<String>,
}