pub mod api_doc;
pub mod controller_utils;
pub mod notification_controller;
pub mod playlist_controller;
pub mod podcast_controller;
pub mod podcast_episode_controller;
pub mod settings_controller;
pub mod sys_info_controller;
pub mod user_controller;
pub mod watch_time_controller;
pub mod web_socket;
pub mod websocket_controller;
pub mod tags_controller;
pub mod server;
pub mod manifest_controller;
mod watch_together;
pub mod watch_together_dto;

pub use watch_together::watch_together_routes;