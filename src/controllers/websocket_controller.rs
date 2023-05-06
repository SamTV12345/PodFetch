use crate::controllers::web_socket::WsConn;
use crate::db::DB;
use crate::models::itunes_models::PodcastEpisode;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix::Addr;
use actix_web::{get, web, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use rss::extension::itunes::{
    ITunesCategory, ITunesCategoryBuilder, ITunesChannelExtensionBuilder,
    ITunesItemExtensionBuilder, ITunesOwner, ITunesOwnerBuilder,
};
use rss::{
    Category, CategoryBuilder, ChannelBuilder, EnclosureBuilder, GuidBuilder, Item, ItemBuilder,
};
use std::sync::Mutex;
use crate::DbPool;
use crate::mutex::LockResultExt;

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

#[get("/rss")]
pub async fn get_rss_feed(
    podcast_episode_service: Data<Mutex<PodcastEpisodeService>>,
) -> HttpResponse {
    let env = EnvironmentService::new();
    let mut podcast_service = podcast_episode_service
        .lock()
        .ignore_poison();
    let downloaded_episodes = podcast_service.find_all_downloaded_podcast_episodes();

    let server_url = env.get_server_url();

    let itunes_owner = get_itunes_owner("Podfetch", "dev@podfetch.com");
    let category = get_category("Technology".to_string());
    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .owner(Some(itunes_owner))
        .categories(vec![category])
        .explicit(Some("no".to_string()))
        .author(Some("Podfetch".to_string()))
        .keywords(Some("Podcast, RSS, Feed".to_string()))
        .new_feed_url(format!("{}{}", &server_url, &"/rss"))
        .summary(Some("Your local rss feed for your podcasts".to_string()))
        .build();

    let channel = ChannelBuilder::default()
        .language("en".to_string())
        .title("Podfetch")
        .link(format!("{}{}", &server_url, &"rss"))
        .description("Your local rss feed for your podcasts")
        .itunes_ext(itunes_ext)
        .items(get_podcast_items_rss(downloaded_episodes))
        .build();

    HttpResponse::Ok().body(channel.to_string())
}

#[get("/rss/{id}")]
pub async fn get_rss_feed_for_podcast(
    podcast_episode_service: Data<Mutex<PodcastEpisodeService>>,
    id: web::Path<i32>,
    conn: Data<DbPool>,
) -> HttpResponse {
    let env = EnvironmentService::new();
    let server_url = env.server_url;
    let mut podcast_service = podcast_episode_service
        .lock()
        .ignore_poison();
    let res = DB::get_podcast(&mut conn.get().unwrap(),id.clone());

    match res {
        Ok(podcast) => {
            let downloaded_episodes =
                podcast_service.find_all_downloaded_podcast_episodes_by_podcast_id(id.clone());

            let mut itunes_owner = get_itunes_owner("", "");

            if podcast.author.is_some() {
                itunes_owner =
                    get_itunes_owner(&podcast.clone().author.unwrap(), "local@local.com");
            }

            let mut categories: Vec<Category> = vec![];
            if podcast.keywords.is_some() {
                let keywords = podcast.clone().keywords.unwrap();
                let keywords: Vec<String> = keywords.split(",").map(|s| s.to_string()).collect();
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
                        .unwrap()
                        .split(",")
                        .map(|s| s.to_string())
                        .collect(),
                ))
                .explicit(podcast.clone().explicit)
                .author(podcast.clone().author)
                .keywords(podcast.clone().keywords)
                .new_feed_url(format!("{}{}/{}", &server_url, &"/rss", &id))
                .summary(podcast.summary.clone())
                .build();

            let channel = ChannelBuilder::default()
                .language(podcast.language)
                .categories(categories)
                .title(podcast.name)
                .link(format!("{}{}/{}", &server_url, &"rss", &id))
                .description(podcast.summary.unwrap())
                .itunes_ext(itunes_ext)
                .items(get_podcast_items_rss(downloaded_episodes))
                .build();

            HttpResponse::Ok().body(channel.to_string())
        }
        Err(..) => {
            return HttpResponse::NotFound().finish();
        }
    }
}

fn get_podcast_items_rss(downloaded_episodes: Vec<PodcastEpisode>) -> Vec<Item> {
    let env = EnvironmentService::new();
    let server_url = env.server_url;

    downloaded_episodes
        .iter()
        .map(|episode| {
            let enclosure = EnclosureBuilder::default()
                .url(format!("{}{}",server_url,&episode.clone().local_url))
                .length(episode.clone().total_time.to_string())
                .mime_type(format!(
                    "{}/{}",
                    "audio",
                    &*PodcastEpisodeService::get_url_file_suffix(&episode.clone().local_url)
                ))
                .build();

            let itunes_extension = ITunesItemExtensionBuilder::default()
                .duration(Some(episode.clone().total_time.to_string()))
                .image(Some(format!("{}{}",server_url,episode.clone().local_image_url)))
                .build();

            let guid = GuidBuilder::default()
                .permalink(false)
                .value(episode.clone().episode_id)
                .build();
            let item = ItemBuilder::default()
                .guid(Some(guid))
                .title(Some(episode.clone().name))
                .description(Some(episode.clone().description))
                .enclosure(Some(enclosure))
                .itunes_ext(itunes_extension)
                .build();
            return item;
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
