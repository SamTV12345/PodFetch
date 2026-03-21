use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::controllers::playlist_controller::PlaylistDto;
use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::models::episode::Episode;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::playlist::Playlist;
use crate::models::playlist_item::PlaylistItem;
use crate::models::podcast_episode::PodcastEpisode;
use crate::utils::error::CustomError;
use podfetch_domain::user::User;
use podfetch_web::playlist::{PlaylistApplicationService, PlaylistDtoPost};

#[derive(Clone, Default)]
pub struct PlaylistService;

impl PlaylistService {
    pub fn new() -> Self {
        Self
    }

    fn map_user(user_id: i32, username: String) -> User {
        User::new(
            user_id,
            username,
            "user",
            None::<String>,
            chrono::Utc::now().naive_utc(),
            true,
        )
    }

    fn to_playlist_dto(
        playlist: Playlist,
        items: Vec<(PlaylistItem, PodcastEpisode, Option<Episode>)>,
        user: User,
    ) -> PlaylistDto {
        let items = items
            .into_iter()
            .map(
                |(_, podcast_episode, history): (PlaylistItem, PodcastEpisode, Option<Episode>)| {
                    PodcastEpisodeWithHistory {
                        podcast_episode: <(
                            PodcastEpisode,
                            Option<User>,
                            Option<FavoritePodcastEpisode>,
                        ) as Into<PodcastEpisodeDto>>::into(
                            (
                            podcast_episode,
                            Some(user.clone()),
                            None,
                        )
                        ),
                        podcast_history_item: history
                            .map(|episode: Episode| episode.convert_to_episode_dto()),
                    }
                },
            )
            .collect();

        PlaylistDto {
            id: playlist.id,
            name: playlist.name,
            items,
        }
    }
}

impl PlaylistApplicationService for PlaylistService {
    type Error = CustomError;
    type PlaylistDto = PlaylistDto;

    fn add_playlist(
        &self,
        user_id: i32,
        username: String,
        playlist: PlaylistDtoPost,
    ) -> Result<Self::PlaylistDto, Self::Error> {
        let user = Self::map_user(user_id, username);
        let playlist = Playlist::create_new_playlist(playlist, user.clone())?;
        let items = PlaylistItem::get_playlist_items_by_playlist_id(playlist.id.clone(), &user)?;
        Ok(Self::to_playlist_dto(playlist, items, user))
    }

    fn update_playlist(
        &self,
        user_id: i32,
        username: String,
        playlist_id: String,
        playlist: PlaylistDtoPost,
    ) -> Result<Self::PlaylistDto, Self::Error> {
        let user = Self::map_user(user_id, username);
        let playlist = Playlist::update_playlist(playlist, playlist_id, user.clone())?;
        let items = PlaylistItem::get_playlist_items_by_playlist_id(playlist.id.clone(), &user)?;
        Ok(Self::to_playlist_dto(playlist, items, user))
    }

    fn get_all_playlists(
        &self,
        user_id: i32,
        username: String,
    ) -> Result<Vec<Self::PlaylistDto>, Self::Error> {
        let user = Self::map_user(user_id, username);
        Playlist::get_playlists(user.id)?
            .into_iter()
            .map(|playlist| {
                let items =
                    PlaylistItem::get_playlist_items_by_playlist_id(playlist.id.clone(), &user)?;
                Ok(Self::to_playlist_dto(playlist, items, user.clone()))
            })
            .collect()
    }

    fn get_playlist_by_id(
        &self,
        user_id: i32,
        username: String,
        playlist_id: String,
    ) -> Result<Self::PlaylistDto, Self::Error> {
        let user = Self::map_user(user_id, username);
        let playlist = Playlist::get_playlist_by_user_and_id(playlist_id.clone(), user.clone())?;
        let items = PlaylistItem::get_playlist_items_by_playlist_id(playlist_id, &user)?;
        Ok(Self::to_playlist_dto(playlist, items, user))
    }

    fn delete_playlist_by_id(&self, user_id: i32, playlist_id: String) -> Result<(), Self::Error> {
        Playlist::delete_playlist_by_id(playlist_id, user_id)
    }

    fn delete_playlist_item(
        &self,
        user_id: i32,
        playlist_id: String,
        episode_id: i32,
    ) -> Result<(), Self::Error> {
        Playlist::delete_playlist_item(playlist_id, episode_id, user_id)
    }
}
