use crate::adapters::file::file_handler::{FileHandlerType, resolve_file_handler_type};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::podcast::Podcast;
use podfetch_domain::user::User;
use podfetch_web::podcast::PodcastDto;
use podfetch_web::tags::Tag;
use std::collections::HashSet;

pub fn map_podcast_to_dto(value: Podcast) -> PodcastDto {
    let image_url = format!(
        "{}{}",
        ENVIRONMENT_SERVICE.get_server_url(),
        value.image_url
    );
    let keywords = dedupe_keywords(value.keywords.clone());
    let podfetch_rss_feed = build_podfetch_feed(value.id, None);

    PodcastDto {
        id: value.id,
        name: value.name.clone(),
        directory_id: value.directory_id.clone(),
        rssfeed: value.rssfeed.clone(),
        image_url,
        language: value.language.clone(),
        keywords,
        podfetch_feed: podfetch_rss_feed,
        summary: value.summary.clone(),
        explicit: value.explicit.clone(),
        last_build_date: value.last_build_date.clone(),
        author: value.author.clone(),
        active: value.active,
        original_image_url: value.original_image_url.clone(),
        directory_name: value.directory_name.clone(),
        tags: vec![],
        favorites: false,
    }
}

pub fn map_podcast_with_context_to_dto(
    value: Podcast,
    favorite: Option<bool>,
    tags: Vec<Tag>,
    user: &User,
) -> PodcastDto {
    let image_url = match resolve_file_handler_type(value.download_location.clone()) {
        FileHandlerType::Local => {
            format!(
                "{}{}",
                ENVIRONMENT_SERVICE.get_server_url(),
                value.image_url
            )
        }
        FileHandlerType::S3 => {
            format!(
                "{}/{}",
                ENVIRONMENT_SERVICE.s3_config.endpoint.clone(),
                &value.image_url
            )
        }
    };

    PodcastDto {
        id: value.id,
        name: value.name.clone(),
        directory_id: value.directory_id.clone(),
        rssfeed: value.rssfeed.clone(),
        image_url,
        podfetch_feed: build_podfetch_feed(value.id, user.api_key.as_deref()),
        language: value.language.clone(),
        keywords: dedupe_keywords(value.keywords.clone()),
        summary: value.summary.clone(),
        explicit: value.explicit.clone(),
        last_build_date: value.last_build_date.clone(),
        author: value.author.clone(),
        active: value.active,
        original_image_url: value.original_image_url.clone(),
        directory_name: value.directory_name.clone(),
        tags,
        favorites: favorite.unwrap_or(false),
    }
}

fn dedupe_keywords(keywords: Option<String>) -> Option<String> {
    keywords.map(|k| {
        k.split(',')
            .map(|keyword| keyword.trim().to_string())
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",")
    })
}

fn build_podfetch_feed(podcast_id: i32, api_key: Option<&str>) -> String {
    let mut podfetch_rss_feed = ENVIRONMENT_SERVICE.build_url_to_rss_feed();
    podfetch_rss_feed
        .join(&format!("/{}", podcast_id))
        .expect("safe string join for podcast rss feed");

    if let Some(api_key) = api_key {
        podfetch_rss_feed
            .query_pairs_mut()
            .append_pair("apiKey", api_key);
    }

    podfetch_rss_feed.to_string()
}

#[cfg(test)]
mod tests {
    use super::map_podcast_to_dto;
    use crate::test_utils::test_builder::podcast_test_builder::tests::PodcastTestDataBuilder;

    #[test]
    fn test_podcast_2_keywords() {
        let podcast = PodcastTestDataBuilder::default()
            .keywords("keyword1, keyword2, keyword1".to_string())
            .build()
            .unwrap()
            .build();
        let podcast_dto = map_podcast_to_dto(podcast.into());
        assert_eq!(podcast_dto.keywords.unwrap().split(',').count(), 2);
    }

    #[test]
    fn test_podcast_3_keywords() {
        let podcast = PodcastTestDataBuilder::default()
            .keywords("keyword1, keyword2, keyword1, keyword2, keyword1, keyword3".to_string())
            .build()
            .unwrap()
            .build();
        let podcast_dto = map_podcast_to_dto(podcast.into());
        assert_eq!(podcast_dto.keywords.unwrap().split(',').count(), 3);
    }
}

