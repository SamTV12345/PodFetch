#[derive(Deserialize, Serialize, Debug)]
pub struct PodcastFavorUpdateModel {
    pub id: i32,
    pub favored: bool,
}