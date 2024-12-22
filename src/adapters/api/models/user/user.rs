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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub explicit_consent: bool,
    pub created_at: NaiveDateTime,
    pub api_key: Option<String>,
}