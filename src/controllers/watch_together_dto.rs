#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherDto {
    pub room_id: String,
    pub admin: String,
    pub room_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherDtoCreate {
    pub room_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherDtoDelete {
    pub room_id: String,
}

pub struct WatchTogetherDtoUpdate {
    pub room_id: String,
    pub room_name: String,
}

pub struct WatchTogetherUsersDTO {}
