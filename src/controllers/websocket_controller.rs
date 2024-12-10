use crate::controllers::web_socket::{chat_ws};

use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;

use crate::service::environment_service::EnvironmentService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::CustomError;
use actix_web::web::Query;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use rss::extension::itunes::{
    ITunesCategory, ITunesCategoryBuilder, ITunesChannelExtension, ITunesChannelExtensionBuilder,
    ITunesItemExtensionBuilder, ITunesOwner, ITunesOwnerBuilder,
};
use rss::{
    Category, CategoryBuilder, Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item,
    ItemBuilder,
};
use tokio::task::spawn_local;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::controllers::server::ChatServerHandle;
use crate::models::user::User;

#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Gets a web socket connection"))
, tag = "info")]
#[get("/ws")]
pub async fn start_connection(
    req: HttpRequest,
    body: web::Payload,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, body)?;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(chat_ws(
        (**chat_server).clone(),
        session,
        msg_stream,
    ));

    Ok(res)
}

#[derive(Deserialize, Serialize)]
pub struct RSSQuery {
    top: i32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RSSAPiKey {
    pub api_key: String,
}

#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Gets the complete rss feed"))
, tag = "info")]
#[get("/rss")]
pub async fn get_rss_feed(
    query: Option<Query<RSSQuery>>,
    api_key: Option<Query<RSSAPiKey>>,
) -> Result<HttpResponse, CustomError> {
    use crate::ENVIRONMENT_SERVICE;

    let env = ENVIRONMENT_SERVICE.get().unwrap();
    // If http basic is enabled, we need to check if the api key is valid
    if env.http_basic || env.oidc_configured {
        if api_key.is_none() {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
        let api_key = api_key.as_ref().unwrap().api_key.to_string();

        let api_key_exists = User::check_if_api_key_exists(api_key);

        if !&api_key_exists {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
    }

    let downloaded_episodes = match query {
        Some(q) => PodcastEpisodeService::find_all_downloaded_podcast_episodes_with_top_k(
            q.top,
        )?,
        None => PodcastEpisodeService::find_all_downloaded_podcast_episodes(
            env,
        )?,
    };

    let server_url = env.get_server_url();
    let feed_url = add_api_key_to_url(format!("{}{}", &server_url, &"rss"), &api_key);

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

    let items = get_podcast_items_rss(downloaded_episodes.clone(), &api_key);

    let channel_builder = ChannelBuilder::default()
        .language("en".to_string())
        .title("Podfetch")
        .link(feed_url)
        .description("Your local rss feed for your podcasts")
        .items(items.clone())
        .clone();

    let channel =
        generate_itunes_extension_conditionally(itunes_ext, channel_builder, None, env, &api_key);

    Ok(HttpResponse::Ok().body(channel.to_string()))
}

fn add_api_key_to_url(url: String, api_key: &Option<Query<RSSAPiKey>>) -> String {
    if let Some(ref api_key) = api_key {
        if url.contains('?') {
            return format!("{}&apiKey={}", url, api_key.api_key);
        }
        return format!("{}?apiKey={}", url, api_key.api_key);
    }
    url
}

fn generate_itunes_extension_conditionally(
    mut itunes_ext: ITunesChannelExtension,
    mut channel_builder: ChannelBuilder,
    podcast: Option<Podcast>,
    env: &EnvironmentService,
    api_key: &Option<Query<RSSAPiKey>>,
) -> Channel {
    if let Some(e) = podcast {
        match !e.image_url.is_empty() {
            true => itunes_ext.set_image(add_api_key_to_url(
                env.server_url.to_string() + &*e.image_url,
                api_key,
            )),
            false => itunes_ext.set_image(add_api_key_to_url(
                env.server_url.to_string() + &*e.original_image_url,
                api_key,
            )),
        }
    }

    channel_builder.itunes_ext(itunes_ext).build()
}

#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Gets a specific rss feed"))
, tag = "info")]
#[get("/rss/{id}")]
pub async fn get_rss_feed_for_podcast(
    id: web::Path<i32>,
    api_key: Option<web::Query<RSSAPiKey>>,
) -> Result<HttpResponse, CustomError> {
    let env = ENVIRONMENT_SERVICE.get().unwrap();
    let server_url = env.server_url.clone();

    // If http basic is enabled, we need to check if the api key is valid
    if env.http_basic || env.oidc_configured {
        if api_key.is_none() {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
        let api_key = api_key.as_ref().unwrap().api_key.to_string();

        let api_key_exists =
            User::check_if_api_key_exists(api_key);

        if !api_key_exists {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
    }

    let podcast = Podcast::get_podcast(*id)?;

    let downloaded_episodes =
        PodcastEpisodeService::find_all_downloaded_podcast_episodes_by_podcast_id(
            *id,
        )?;

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

    let items = get_podcast_items_rss(downloaded_episodes.clone(), &api_key);
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
        ENVIRONMENT_SERVICE.get().unwrap(),
        &api_key,
    );

    Ok(HttpResponse::Ok().body(channel.to_string()))
}

fn get_podcast_items_rss(
    downloaded_episodes: Vec<PodcastEpisode>,
    api_key: &Option<Query<RSSAPiKey>>,
) -> Vec<Item> {
    downloaded_episodes
        .iter()
        .map(|episode| {
            let mut episode = episode.clone();
            episode.local_url = add_api_key_to_url(episode.local_url.clone(), api_key);
            episode.local_image_url = add_api_key_to_url(episode.local_image_url.clone(), api_key);

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
