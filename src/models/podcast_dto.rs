use crate::adapters::file::file_handler::FileHandlerType;
use crate::adapters::file::s3_file_handler::S3_BUCKET_CONFIG;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::favorites::Favorite;
use crate::models::podcasts::Podcast;
use crate::models::tag::Tag;
use std::collections::HashSet;
use utoipa::ToSchema;
use crate::models::user::User;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct PodcastDto {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub directory_id: String,
    pub directory_name: String,
    pub podfetch_feed: String,
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

impl From<(Podcast, Option<Favorite>, Vec<Tag>, &User)> for PodcastDto {
    fn from(value: (Podcast, Option<Favorite>, Vec<Tag>, &User)) -> Self {
        let favorite = value.1.is_some() && value.1.clone().unwrap().favored;

        let image_url =
            match FileHandlerType::from(value.0.download_location.clone().unwrap().as_str()) {
                FileHandlerType::Local => {
                    format!(
                        "{}{}",
                        ENVIRONMENT_SERVICE.get_server_url(),
                        value.0.image_url
                    )
                }
                FileHandlerType::S3 => {
                    format!(
                        "{}/{}",
                        S3_BUCKET_CONFIG.endpoint.clone(),
                        &value.0.image_url
                    )
                }
            };

        let keywords_to_map = value.0.keywords.clone();
        let keywords = keywords_to_map.map(|k| {
            k.split(",")
                .map(|k| k.trim().to_string())
                .collect::<HashSet<String>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(",")
        });

        let mut podfetch_rss_feed = ENVIRONMENT_SERVICE.build_url_to_rss_feed();
        podfetch_rss_feed.join(&format!("/{}", value.0.id)).expect
        ("this is \
        safe because we \
        are \
        just joining strings");


        if let Some(api_key) = &value.3.api_key {
            podfetch_rss_feed.query_pairs_mut().append_pair("api_key", api_key);
        }


        PodcastDto {
            id: value.0.id,
            name: value.0.name.clone(),
            directory_id: value.0.directory_id.clone(),
            rssfeed: value.0.rssfeed.clone(),
            image_url,
            podfetch_feed: podfetch_rss_feed.to_string(),
            language: value.0.language.clone(),
            keywords,
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

        let keywords_to_map = value.keywords.clone();
        let keywords = keywords_to_map.map(|k| {
            k.split(",")
                .map(|k| k.trim().to_string())
                .collect::<HashSet<String>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(",")
        });
        let podfetch_rss_feed = ENVIRONMENT_SERVICE.build_url_to_rss_feed();
        podfetch_rss_feed.join(&format!("/{}", value.id)).expect
        ("this is \
        safe \
        because we \
        are just joining strings");

        PodcastDto {
            id: value.id,
            name: value.name.clone(),
            directory_id: value.directory_id.clone(),
            rssfeed: value.rssfeed.clone(),
            image_url,
            language: value.language.clone(),
            keywords,
            podfetch_feed: podfetch_rss_feed.to_string(),
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

#[cfg(test)]
pub mod tests {
    use crate::models::podcast_dto::PodcastDto;
    use crate::models::podcasts::Podcast;
    use crate::utils::test_builder::podcast_test_builder::tests::PodcastTestDataBuilder;

    #[test]
    pub fn test_podcast_2_keywords() {
        let podcast = PodcastTestDataBuilder::default()
            .keywords("keyword1, keyword2, keyword1".to_string())
            .build()
            .unwrap();
        let podcast_dto: PodcastDto = PodcastDto::from(Podcast::from(podcast));
        assert_eq!(podcast_dto.keywords.unwrap().split(",").count(), 2);
    }

    #[test]
    pub fn test_podcast_3_keywords() {
        let podcast = PodcastTestDataBuilder::default()
            .keywords("keyword1, keyword2, keyword1, keyword2, keyword1, keyword3".to_string())
            .build()
            .unwrap();
        let podcast_dto: PodcastDto = PodcastDto::from(Podcast::from(podcast));
        assert_eq!(podcast_dto.keywords.unwrap().split(",").count(), 3);
    }
}
