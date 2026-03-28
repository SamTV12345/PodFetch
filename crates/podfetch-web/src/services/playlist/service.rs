use crate::podcast_episode_dto::PodcastEpisodeDto;
use podfetch_persistence::db::database;
use podfetch_persistence::adapters::PlaylistRepositoryImpl;
use crate::controllers::playlist_controller::PlaylistDto;
use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use podfetch_persistence::episode::EpisodeEntity as Episode;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use common_infrastructure::error::CustomError;
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::playlist::{Playlist, PlaylistItem, PlaylistRepository};
use podfetch_domain::user::User;
use crate::history::map_episode_to_dto;
use crate::playlist::{PlaylistApplicationService, PlaylistDtoPost};
use std::sync::Arc;

#[derive(Clone)]
pub struct PlaylistService {
    repository: Arc<dyn PlaylistRepository<Error = CustomError>>,
}

impl PlaylistService {
    pub fn new(repository: Arc<dyn PlaylistRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(PlaylistRepositoryImpl::new(database())))
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
                            .map(Into::into)
                            .as_ref()
                            .map(map_episode_to_dto),
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

    fn load_playlist_items(
        &self,
        playlist_id: &str,
        user: &User,
    ) -> Result<Vec<(PlaylistItem, PodcastEpisode, Option<Episode>)>, CustomError> {
        let items = self.repository.list_items_by_playlist_id(playlist_id)?;

        items
            .into_iter()
            .filter_map(|item| {
                let podcast_episode =
                    PodcastEpisodeService::get_podcast_episode_by_internal_id(item.episode)
                        .ok()
                        .flatten()?;
                let history =
                    WatchtimeService::get_watchtime(&podcast_episode.episode_id, &user.username)
                        .ok()?;
                Some(Ok((item, podcast_episode, history)))
            })
            .collect()
    }

    fn find_playlist_by_id(&self, playlist_id: &str) -> Result<Playlist, CustomError> {
        self.repository.find_by_id(playlist_id)?.ok_or_else(|| {
            common_infrastructure::error::CustomErrorInner::NotFound(
                common_infrastructure::error::ErrorSeverity::Debug,
            )
            .into()
        })
    }

    fn find_playlist_by_user_and_id(
        &self,
        playlist_id: &str,
        user_id: i32,
    ) -> Result<Playlist, CustomError> {
        self.repository
            .find_by_user_and_id(playlist_id, user_id)?
            .ok_or_else(|| {
                common_infrastructure::error::CustomErrorInner::NotFound(
                    common_infrastructure::error::ErrorSeverity::Debug,
                )
                .into()
            })
    }

    fn create_playlist_if_missing(
        &self,
        playlist: PlaylistDtoPost,
        user: &User,
    ) -> Result<Playlist, CustomError> {
        if let Some(existing) = self.repository.find_by_name(&playlist.name)? {
            return Ok(existing);
        }

        let inserted = self.repository.insert_playlist(Playlist {
            id: uuid::Uuid::new_v4().to_string(),
            name: playlist.name.clone(),
            user_id: user.id,
        })?;

        for (position, item) in playlist.items.iter().enumerate() {
            self.repository.insert_playlist_item(PlaylistItem {
                playlist_id: inserted.id.clone(),
                episode: item.episode,
                position: position as i32,
            })?;
        }

        Ok(inserted)
    }

    pub fn delete_playlist_items_by_episode_id(&self, episode_id: i32) -> Result<(), CustomError> {
        self.repository.delete_items_by_episode_id(episode_id)?;
        Ok(())
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
        let playlist = self.create_playlist_if_missing(playlist, &user)?;
        let items = self.load_playlist_items(&playlist.id, &user)?;
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
        let playlist_to_update = self.find_playlist_by_id(&playlist_id)?;
        if playlist_to_update.user_id != user.id {
            return Err(common_infrastructure::error::CustomErrorInner::Forbidden(
                common_infrastructure::error::ErrorSeverity::Info,
            )
            .into());
        }

        self.repository
            .update_playlist_name(&playlist_id, user.id, &playlist.name)?;
        self.repository.delete_items_by_playlist_id(&playlist_id)?;
        for (position, item) in playlist.items.iter().enumerate() {
            self.repository.insert_playlist_item(PlaylistItem {
                playlist_id: playlist_id.clone(),
                episode: item.episode,
                position: position as i32,
            })?;
        }

        let playlist = self.find_playlist_by_id(&playlist_id)?;
        let items = self.load_playlist_items(&playlist.id, &user)?;
        Ok(Self::to_playlist_dto(playlist, items, user))
    }

    fn get_all_playlists(
        &self,
        user_id: i32,
        username: String,
    ) -> Result<Vec<Self::PlaylistDto>, Self::Error> {
        let user = Self::map_user(user_id, username);
        self.repository
            .list_by_user(user.id)?
            .into_iter()
            .map(|playlist| {
                let items = self.load_playlist_items(&playlist.id, &user)?;
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
        let playlist = self.find_playlist_by_user_and_id(&playlist_id, user.id)?;
        let items = self.load_playlist_items(&playlist_id, &user)?;
        Ok(Self::to_playlist_dto(playlist, items, user))
    }

    fn delete_playlist_by_id(&self, user_id: i32, playlist_id: String) -> Result<(), Self::Error> {
        let playlist = self.find_playlist_by_id(&playlist_id)?;
        if playlist.user_id != user_id {
            return Err(common_infrastructure::error::CustomErrorInner::Forbidden(
                common_infrastructure::error::ErrorSeverity::Info,
            )
            .into());
        }

        self.repository.delete_items_by_playlist_id(&playlist_id)?;
        self.repository.delete_playlist(&playlist_id, user_id)?;
        Ok(())
    }

    fn delete_playlist_item(
        &self,
        user_id: i32,
        playlist_id: String,
        episode_id: i32,
    ) -> Result<(), Self::Error> {
        let playlist = self.find_playlist_by_id(&playlist_id)?;
        if playlist.user_id != user_id {
            return Err(common_infrastructure::error::CustomErrorInner::Forbidden(
                common_infrastructure::error::ErrorSeverity::Warning,
            )
            .into());
        }

        self.repository
            .delete_playlist_item(&playlist_id, episode_id)?;
        Ok(())
    }
}


