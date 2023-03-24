use std::sync::Mutex;
use actix::Addr;
use actix_web::{web::Payload, HttpResponse, Error, HttpRequest, web::Data, get};
use actix_web_actors::ws;
use rss::{ChannelBuilder, EnclosureBuilder, GuidBuilder, Item, ItemBuilder};
use rss::extension::itunes::{ITunesCategoryBuilder, ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder, ITunesOwnerBuilder};
use crate::controllers::web_socket::WsConn;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::podcast_episode_service::PodcastEpisodeService;

#[get("/ws")]
pub async fn start_connection(req: HttpRequest, stream: Payload, lobby: Data<Addr<Lobby>>)
                              -> Result<HttpResponse, Error> {
    let ws = WsConn::new( lobby.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}


#[get("/rss")]
pub async fn get_rss_feed(podcast_episode_service: Data<Mutex<PodcastEpisodeService>>) -> HttpResponse {
    let env = EnvironmentService::new();
    let mut podcast_service = podcast_episode_service
        .lock()
        .expect("Error locking podcast service");
    let downloaded_episodes = podcast_service.find_all_downloaded_podcast_episodes();
    let items = downloaded_episodes.iter().map(|episode|{

        let enclosure = EnclosureBuilder::default()
            .url(&episode.clone().local_url)
            .length(episode.clone().total_time.to_string())
            .mime_type(format!("{}/{}","audio",&*PodcastEpisodeService::get_url_file_suffix(&episode
                .clone()
                .local_url)))
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
            .title(Some(episode.clone().name))
            .description(Some(episode.clone().description))
            .enclosure(Some(enclosure))
            .itunes_ext(itunes_extension)
            .build();
        return item
    }).collect::<Vec<Item>>();
    let server_url = env.get_server_url();

    let itunes_owner = ITunesOwnerBuilder::default()
        .name(Some("Podfetch".to_string()))
        .email(Some("podfetch@podfetch.dev".to_string()))
        .build();
    let category = ITunesCategoryBuilder::default()
        .text("Technology").build();
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
        .items(items)
        .build();

    HttpResponse::Ok().body(channel.to_string())
}