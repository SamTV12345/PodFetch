// decode request data
#[derive(Deserialize)]
pub struct UserData {
    pub username: String,
}
// this is to insert users to database
#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub first_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodCastAddModel {
    pub track_id: i64,
    pub user_id: i64
}