use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteDto {
    pub username: String,
    pub podcast_id: i32,
    pub favored: bool,
}