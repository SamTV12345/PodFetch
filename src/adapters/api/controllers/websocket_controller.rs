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
use crate::adapters::api::ws::server::ChatServerHandle;
use crate::adapters::api::ws::web_socket::chat_ws;

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







