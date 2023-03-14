use actix::prelude::{Message, Recipient};
use uuid::Uuid;
use crate::models::itunes_models::{Podcast, PodcastEpisode};

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub self_id: Uuid,
}

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BroadcastMessage{
    pub type_of: String,
    pub message: String,
    pub podcast: Option<Podcast>,
    pub podcast_episodes: Option<Vec<PodcastEpisode>>,
    pub podcast_episode: Option<PodcastEpisode>
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid
}