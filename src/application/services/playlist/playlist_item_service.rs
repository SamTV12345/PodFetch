use crate::adapters::persistence::repositories::playlist::playlist_item::PlaylistItemRepositoryImpl;
use crate::utils::error::CustomError;

pub struct PlaylistItemService;


impl PlaylistItemService {
    pub fn delete_playlist_items_by_episode_id(episode_id: i32) -> Result<(), CustomError> {
        PlaylistItemRepositoryImpl::delete_playlist_item_by_episode_id(episode_id)
    }
}