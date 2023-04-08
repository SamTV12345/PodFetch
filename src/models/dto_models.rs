use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PodcastFavorUpdateModel {
    pub id: i32,
    pub favored: bool,
}
