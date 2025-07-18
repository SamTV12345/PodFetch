use crate::models::podcasts::Podcast;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::OptionalQuery;

use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::user::User;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use rss::extension::itunes::{
    ITunesCategory, ITunesCategoryBuilder, ITunesChannelExtension, ITunesChannelExtensionBuilder,
    ITunesItemExtensionBuilder, ITunesOwner, ITunesOwnerBuilder,
};
use rss::{
    Category, CategoryBuilder, Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item,
    ItemBuilder,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize, Serialize)]
pub struct RSSQuery {
    top: Option<i32>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RSSAPiKey {
    pub api_key: String,
}

#[utoipa::path(
get,
path="/rss",
responses(
(status = 200, description = "Gets the complete rss feed"))
, tag = "rss")]
pub async fn get_rss_feed(
    OptionalQuery(query): OptionalQuery<RSSQuery>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
) -> Result<impl IntoResponse, CustomError> {
    use crate::ENVIRONMENT_SERVICE;

    // If http basic is enabled, we need to check if the api key is valid
    if ENVIRONMENT_SERVICE.http_basic || ENVIRONMENT_SERVICE.oidc_configured {
        let api_key = match &api_key {
            Some(q) => Ok::<&RSSAPiKey, CustomError>(q),
            None => Err(CustomErrorInner::Forbidden(Warning).into()),
        }?;
        let api_key = &api_key.api_key;

        let api_key_exists = User::check_if_api_key_exists(api_key);

        if !&api_key_exists {
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }
    }

    let downloaded_episodes = match query {
        Some(q) => match q.top {
            Some(q) => PodcastEpisodeService::find_all_downloaded_podcast_episodes_with_top_k(q)?,
            None => PodcastEpisodeService::find_all_downloaded_podcast_episodes()?,
        },
        None => PodcastEpisodeService::find_all_downloaded_podcast_episodes()?,
    };

    let api_key = api_key.map(|c| c.api_key.to_string());

    let downloaded_episodes: Vec<PodcastEpisodeDto> = downloaded_episodes
        .into_iter()
        .map(|c| (c, api_key.clone(), None::<FavoritePodcastEpisode>).into())
        .collect();

    let feed_url = add_api_key_to_url(
        format!("{}{}", &ENVIRONMENT_SERVICE.server_url, &"rss"),
        &api_key,
    );
    let itunes_owner = get_itunes_owner("Podfetch", "dev@podfetch.com");
    let category = get_category("Technology".to_string());
    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .owner(Some(itunes_owner))
        .categories(vec![category])
        .explicit(Some("no".to_string()))
        .author(Some("Podfetch".to_string()))
        .keywords(Some("Podcast, RSS, Feed".to_string()))
        .new_feed_url(feed_url.clone())
        .summary(Some("Your local rss feed for your podcasts".to_string()))
        .build();

    let items = get_podcast_items_rss(&downloaded_episodes);

    let channel_builder = ChannelBuilder::default()
        .language("en".to_string())
        .title("Podfetch")
        .link(feed_url)
        .description("Your local rss feed for your podcasts")
        .items(items.clone())
        .clone();

    let channel =
        generate_itunes_extension_conditionally(itunes_ext, channel_builder, None, &api_key);

    let response = Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(channel.to_string())
        .unwrap();
    Ok(response)
}

fn add_api_key_to_url(url: String, api_key: &Option<String>) -> String {
    if let Some(ref api_key) = api_key {
        if url.contains('?') {
            return format!("{url}&apiKey={api_key}");
        }
        return format!("{url}?apiKey={api_key}");
    }
    url
}

fn generate_itunes_extension_conditionally(
    mut itunes_ext: ITunesChannelExtension,
    mut channel_builder: ChannelBuilder,
    podcast: Option<Podcast>,
    api_key: &Option<String>,
) -> Channel {
    if let Some(e) = podcast {
        match !e.image_url.is_empty() {
            true => itunes_ext.set_image(add_api_key_to_url(
                ENVIRONMENT_SERVICE.server_url.to_string() + &*e.image_url,
                api_key,
            )),
            false => itunes_ext.set_image(add_api_key_to_url(
                ENVIRONMENT_SERVICE.server_url.to_string() + &*e.original_image_url,
                api_key,
            )),
        }
    }

    channel_builder.itunes_ext(itunes_ext).build()
}

#[utoipa::path(
get,
path="/rss/{id}",
responses(
(status = 200, description = "Gets a specific rss feed"))
, tag = "rss")]
pub async fn get_rss_feed_for_podcast(
    Path(id): Path<i32>,
    OptionalQuery(api_key): OptionalQuery<RSSAPiKey>,
) -> Result<impl IntoResponse, CustomError> {
    let server_url = ENVIRONMENT_SERVICE.server_url.clone();
    // If http basic is enabled, we need to check if the api key is valid
    if ENVIRONMENT_SERVICE.http_basic || ENVIRONMENT_SERVICE.oidc_configured {
        let api_key = match &api_key {
            Some(q) => Ok::<&RSSAPiKey, CustomError>(q),
            None => Err(CustomErrorInner::Forbidden(Warning).into()),
        }?;
        let api_key = &api_key.api_key;

        let api_key_exists = User::check_if_api_key_exists(api_key);

        if !&api_key_exists {
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }
    }
    let api_key = api_key.map(|c| c.api_key.clone());
    let podcast = Podcast::get_podcast(id)?;

    let downloaded_episodes: Vec<PodcastEpisodeDto> =
        PodcastEpisodeService::find_all_downloaded_podcast_episodes_by_podcast_id(id)?
            .into_iter()
            .map(|c| (c, api_key.clone(), None::<FavoritePodcastEpisode>).into())
            .collect();

    let mut itunes_owner = get_itunes_owner("", "");

    if let Some(author) = podcast.author.clone() {
        itunes_owner = get_itunes_owner(&author, "local@local.com")
    }

    let mut categories: Vec<Category> = vec![];
    if let Some(keyword) = podcast.keywords.clone() {
        let keywords: Vec<String> = keyword.split(',').map(|s| s.to_string()).collect();
        categories = keywords
            .iter()
            .map(|keyword| CategoryBuilder::default().name(keyword).build())
            .collect();
    }

    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .owner(Some(itunes_owner))
        .categories(get_categories(
            podcast
                .clone()
                .keywords
                .clone()
                .unwrap()
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        ))
        .explicit(podcast.clone().explicit)
        .author(podcast.clone().author)
        .keywords(podcast.clone().keywords)
        .new_feed_url(add_api_key_to_url(
            format!("{}{}/{}", &server_url, &"rss", &id),
            &api_key,
        ))
        .summary(podcast.summary.clone())
        .build();

    let items = get_podcast_items_rss(&downloaded_episodes);
    let channel_builder = ChannelBuilder::default()
        .language(podcast.clone().language)
        .categories(categories)
        .title(podcast.name.clone())
        .link(add_api_key_to_url(
            format!("{}{}/{}", &server_url, &"rss", &id),
            &api_key,
        ))
        .description(podcast.clone().summary.unwrap())
        .items(items.clone())
        .clone();

    let channel = generate_itunes_extension_conditionally(
        itunes_ext,
        channel_builder,
        Some(podcast.clone()),
        &api_key,
    );
    let response = Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(channel.to_string())
        .unwrap();
    Ok(response)
}

fn get_podcast_items_rss(downloaded_episodes: &[PodcastEpisodeDto]) -> Vec<Item> {
    downloaded_episodes
        .iter()
        .map(|episode| {
            let episode = episode.clone();

            let enclosure = EnclosureBuilder::default()
                .url(episode.local_url.clone())
                .length(episode.clone().total_time.to_string())
                .mime_type(format!(
                    "audio/{}",
                    PodcastEpisodeService::get_url_file_suffix(&episode.clone().local_url).unwrap()
                ))
                .build();

            let itunes_extension = ITunesItemExtensionBuilder::default()
                .duration(Some(episode.clone().total_time.to_string()))
                .image(Some(episode.clone().local_image_url))
                .build();

            let guid = GuidBuilder::default()
                .permalink(false)
                .value(episode.clone().episode_id)
                .build();
            let item = ItemBuilder::default()
                .guid(Some(guid))
                .pub_date(Some(episode.clone().date_of_recording))
                .title(Some(episode.clone().name))
                .description(Some(episode.clone().description))
                .enclosure(Some(enclosure))
                .itunes_ext(itunes_extension)
                .build();
            item
        })
        .collect::<Vec<Item>>()
}

fn get_categories(categories: Vec<String>) -> Vec<ITunesCategory> {
    categories
        .iter()
        .map(|category| get_category(category.to_string()))
        .collect::<Vec<ITunesCategory>>()
}

fn get_category(category: String) -> ITunesCategory {
    ITunesCategoryBuilder::default().text(category).build()
}

fn get_itunes_owner(name: &str, email: &str) -> ITunesOwner {
    ITunesOwnerBuilder::default()
        .name(Some(name.to_string()))
        .email(Some(email.to_string()))
        .build()
}

pub fn get_websocket_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_rss_feed))
        .routes(routes!(get_rss_feed_for_podcast))
}
