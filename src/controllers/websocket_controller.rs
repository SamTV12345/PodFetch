use crate::controllers::web_socket::WsConn;
use std::ops::DerefMut;

use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::web_socket_message::Lobby;
use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::DbPool;
use actix::Addr;
use actix_web::{get, web, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use rss::extension::itunes::{
    ITunesCategory, ITunesCategoryBuilder, ITunesChannelExtension, ITunesChannelExtensionBuilder,
    ITunesItemExtensionBuilder, ITunesOwner, ITunesOwnerBuilder,
};
use rss::{
    Category, CategoryBuilder, Channel, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item,
    ItemBuilder,
};
use std::sync::Mutex;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::user::User;

#[utoipa::path(
context_path = "/api/v1",
responses(
(status = 200, description = "Gets a web socket connection"))
, tag = "info")]
#[get("/ws")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    lobby: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let ws = WsConn::new(lobby.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
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
    db: Data<DbPool>,
    query: Option<web::Query<RSSQuery>>,
    api_key: Option<web::Query<RSSAPiKey>>
    ) -> Result<HttpResponse, CustomError> {
    use crate::ENVIRONMENT_SERVICE;

    let env = ENVIRONMENT_SERVICE.get().unwrap();
    // If http basic is enabled, we need to check if the api key is valid
    if env.http_basic || env.oidc_configured {
        if api_key.is_none() {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
        let api_key = api_key.unwrap().api_key.to_string();


        let api_key_exists = User::check_if_api_key_exists(api_key, db.get().unwrap().deref_mut());

        if !api_key_exists {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
    }



    let downloaded_episodes = match query {
        Some(q) => PodcastEpisodeService::find_all_downloaded_podcast_episodes_with_top_k(&mut db.get().unwrap(), q.top)?,
        None => PodcastEpisodeService::find_all_downloaded_podcast_episodes(&mut db.get().unwrap(), env)?,
    };

    let server_url = env.get_server_url();

    let itunes_owner = get_itunes_owner("Podfetch", "dev@podfetch.com");
    let category = get_category("Technology".to_string());
    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .owner(Some(itunes_owner))
        .categories(vec![category])
        .explicit(Some("no".to_string()))
        .author(Some("Podfetch".to_string()))
        .keywords(Some("Podcast, RSS, Feed".to_string()))
        .new_feed_url(format!("{}{}", &server_url, &"rss"))
        .summary(Some("Your local rss feed for your podcasts".to_string()))
        .build();

    let items = get_podcast_items_rss(downloaded_episodes.clone());

    let channel_builder = ChannelBuilder::default()
        .language("en".to_string())
        .title("Podfetch")
        .link(format!("{}{}", &server_url, &"rss"))
        .description("Your local rss feed for your podcasts")
        .items(items.clone())
        .clone();

    let channel =
        generate_itunes_extension_conditionally(itunes_ext, channel_builder, None, env);

    Ok(HttpResponse::Ok().body(channel.to_string()))
}

fn generate_itunes_extension_conditionally(
    mut itunes_ext: ITunesChannelExtension,
    mut channel_builder: ChannelBuilder,
    podcast: Option<Podcast>,
    env: &EnvironmentService,
) -> Channel {
    if let Some(e) = podcast {
        match !e.image_url.is_empty() {
            true => itunes_ext.set_image(env.server_url.to_string() + &*e.image_url),
            false => itunes_ext.set_image(env.server_url.to_string() + &*e.original_image_url),
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
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let server_url = ENVIRONMENT_SERVICE.get().unwrap().server_url.clone();
    let podcast = Podcast::get_podcast(conn.get().map_err(map_r2d2_error)?.deref_mut(), *id)?;

    let downloaded_episodes = PodcastEpisodeService::find_all_downloaded_podcast_episodes_by_podcast_id(
        *id,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
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
        .new_feed_url(format!("{}{}/{}", &server_url, &"rss", &id))
        .summary(podcast.summary.clone())
        .build();

    let items = get_podcast_items_rss(downloaded_episodes.clone());
    let channel_builder = ChannelBuilder::default()
        .language(podcast.clone().language)
        .categories(categories)
        .title(podcast.name.clone())
        .link(format!("{}{}/{}", &server_url, &"rss", &id))
        .description(podcast.clone().summary.unwrap())
        .items(items.clone())
        .clone();

    let channel = generate_itunes_extension_conditionally(
        itunes_ext,
        channel_builder,
        Some(podcast.clone()),
        ENVIRONMENT_SERVICE.get().unwrap()
    );

    Ok(HttpResponse::Ok().body(channel.to_string()))
}

fn get_podcast_items_rss(downloaded_episodes: Vec<PodcastEpisode>) -> Vec<Item> {
    downloaded_episodes
        .iter()
        .map(|episode| {
            let enclosure = EnclosureBuilder::default()
                .url(&episode.clone().local_url)
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
