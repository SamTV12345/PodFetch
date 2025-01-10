use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::favorites::Favorite;
use crate::models::podcasts::Podcast;
use crate::models::tag::Tag;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct PodcastDto {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub directory_id: String,
    pub directory_name: String,
    pub(crate) rssfeed: String,
    pub image_url: String,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<String>,
    pub keywords: Option<String>,
    pub last_build_date: Option<String>,
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    pub favorites: bool,
    pub tags: Vec<Tag>,
}

impl From<(Podcast, Option<Favorite>, Vec<Tag>)> for PodcastDto {
    fn from(value: (Podcast, Option<Favorite>, Vec<Tag>)) -> Self {
        let favorite = value.1.is_some() && value.1.clone().unwrap().favored;
        let image_url = format!(
            "{}{}",
            ENVIRONMENT_SERVICE.get_server_url(),
            value.0.image_url
        );

        PodcastDto {
            id: value.0.id,
            name: value.0.name.clone(),
            directory_id: value.0.directory_id.clone(),
            rssfeed: value.0.rssfeed.clone(),
            image_url,
            language: value.0.language.clone(),
            keywords: value.0.keywords.clone(),
            summary: value.0.summary.clone(),
            explicit: value.0.clone().explicit,
            last_build_date: value.0.clone().last_build_date,
            author: value.0.author.clone(),
            active: value.0.active,
            original_image_url: value.0.original_image_url.clone(),
            directory_name: value.0.directory_name.clone(),
            tags: value.2,
            favorites: favorite,
        }
    }
}

// Used when we don't need the other information
impl From<Podcast> for PodcastDto {
    fn from(value: Podcast) -> Self {
        let image_url = format!(
            "{}{}",
            ENVIRONMENT_SERVICE.get_server_url(),
            value.image_url
        );
        PodcastDto {
            id: value.id,
            name: value.name.clone(),
            directory_id: value.directory_id.clone(),
            rssfeed: value.rssfeed.clone(),
            image_url,
            language: value.language.clone(),
            keywords: value.keywords.clone(),
            summary: value.summary.clone(),
            explicit: value.clone().explicit,
            last_build_date: value.clone().last_build_date,
            author: value.author.clone(),
            active: value.active,
            original_image_url: value.original_image_url.clone(),
            directory_name: value.directory_name.clone(),
            tags: vec![],
            favorites: false,
        }
    }
}
