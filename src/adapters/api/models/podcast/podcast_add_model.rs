use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastAddModel {
    pub track_id: i32,
    pub user_id: i32,
}