use utoipa::ToSchema;
use crate::adapters::api::models::podcast::tag_dto::TagDto;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::podcast::podcast::Podcast;
use crate::domain::models::tag::tag::Tag;
use crate::service::environment_service;

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
    pub tags: Vec<TagDto>,
}

impl From<(Podcast, Vec<Tag>)> for PodcastDto {
    fn from(value: (Podcast, Vec<Tag>)) -> Self {
        Self {
            id: value.0.id,
            name: value.0.name,
            directory_id: value.0.directory_id,
            rssfeed: value.0.rssfeed,
            image_url: environment_service::EnvironmentService::get_server_url(
                ENVIRONMENT_SERVICE.get().unwrap(),
            ) + &value.0.image_url,
            language: value.0.language,
            keywords: value.0.keywords,
            summary: value.0.summary,
            explicit: value.0.explicit,
            last_build_date: value.0.last_build_date,
            author: value.0.author,
            active: value.0.active,
            original_image_url: value.0.original_image_url,
            directory_name: value.0.directory_name,
            tags: value.1.into_iter().map(|tag| tag.into()).collect(),
            favorites: false
        }
    }
}


impl From<(Podcast, Option<Favorite>, Vec<Tag>)> for PodcastDto {
    fn from(value: (Podcast, Option<Favorite>, Vec<Tag>)) -> Self {
        let favorite = if let Some(v) = value.1 {
            v.favored
        } else {
            false
        };
        PodcastDto {
            id: value.0.id,
            name: value.0.name,
            directory_id: value.0.directory_id,
            rssfeed: value.0.rssfeed,
            image_url: environment_service::EnvironmentService::get_server_url(
                ENVIRONMENT_SERVICE.get().unwrap(),
            ) + &value.0.image_url,
            language: value.0.language,
            keywords: value.0.keywords,
            summary: value.0.summary,
            explicit: value.0.explicit,
            last_build_date: value.0.last_build_date,
            author: value.0.author,
            active: value.0.active,
            original_image_url: value.0.original_image_url,
            favorites: favorite,
            directory_name: value.0.directory_name,
            tags: value.2.into_iter().map(|tag| tag.into()).collect(),
        }


    }
}