use crate::db::Database;
use common_infrastructure::error::CustomError;

// ── DeviceSyncGroup ─────────────────────────────────────────────────────────

use crate::device_sync_group::DieselDeviceSyncGroupRepository;
use podfetch_domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};

pub struct DeviceSyncGroupRepositoryImpl {
    inner: DieselDeviceSyncGroupRepository,
}

impl DeviceSyncGroupRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselDeviceSyncGroupRepository::new(database),
        }
    }
}

impl DeviceSyncGroupRepository for DeviceSyncGroupRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id(&self, user_id: i32) -> Result<Vec<DeviceSyncGroup>, Self::Error> {
        self.inner.get_by_user_id(user_id).map_err(Into::into)
    }

    fn replace_all(&self, user_id: i32, groups: Vec<DeviceSyncGroup>) -> Result<(), Self::Error> {
        self.inner.replace_all(user_id, groups).map_err(Into::into)
    }
}

// ── Device ──────────────────────────────────────────────────────────────────

use crate::device::DieselDeviceRepository;
use podfetch_domain::device::{Device, DeviceRepository};

pub struct DeviceRepositoryImpl {
    inner: DieselDeviceRepository,
}

impl DeviceRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselDeviceRepository::new(database),
        }
    }
}

impl DeviceRepository for DeviceRepositoryImpl {
    type Error = CustomError;

    fn create(&self, device: Device) -> Result<Device, CustomError> {
        self.inner.create(device).map_err(Into::into)
    }

    fn get_devices_of_user(&self, user_id_to_find: i32) -> Result<Vec<Device>, CustomError> {
        self.inner
            .get_devices_of_user(user_id_to_find)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: i32) -> Result<(), CustomError> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn list_castable_for_user(&self, user_id: i32) -> Result<Vec<Device>, CustomError> {
        self.inner
            .list_castable_for_user(user_id)
            .map_err(Into::into)
    }
}

// ── Filter ──────────────────────────────────────────────────────────────────

use crate::filter::DieselFilterRepository;
use podfetch_domain::filter::{Filter, FilterRepository};

pub struct FilterRepositoryImpl {
    inner: DieselFilterRepository,
}

impl FilterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselFilterRepository::new(database),
        }
    }
}

impl FilterRepository for FilterRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id(&self, user_id: i32) -> Result<Option<Filter>, Self::Error> {
        self.inner.get_by_user_id(user_id).map_err(Into::into)
    }

    fn save(&self, filter: Filter) -> Result<(), Self::Error> {
        self.inner.save(filter).map_err(Into::into)
    }

    fn save_timeline_decision(&self, user_id: i32, only_favored: bool) -> Result<(), Self::Error> {
        self.inner
            .save_timeline_decision(user_id, only_favored)
            .map_err(Into::into)
    }
}

// ── GpodderSetting ──────────────────────────────────────────────────────────

use crate::gpodder_setting::DieselGpodderSettingRepository;
use podfetch_domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};

pub struct GpodderSettingRepositoryImpl {
    inner: DieselGpodderSettingRepository,
}

impl GpodderSettingRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselGpodderSettingRepository::new(database),
        }
    }
}

impl GpodderSettingRepository for GpodderSettingRepositoryImpl {
    type Error = CustomError;

    fn get_setting(
        &self,
        user_id: i32,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error> {
        self.inner
            .get_setting(user_id, scope, scope_id)
            .map_err(Into::into)
    }

    fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, Self::Error> {
        self.inner.save_setting(setting).map_err(Into::into)
    }
}

// ── Invite ──────────────────────────────────────────────────────────────────

use crate::invite::DieselInviteRepository;
use podfetch_domain::invite::{Invite, InviteRepository};

pub struct InviteRepositoryImpl {
    inner: DieselInviteRepository,
}

impl InviteRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselInviteRepository::new(database),
        }
    }
}

impl InviteRepository for InviteRepositoryImpl {
    type Error = CustomError;

    fn create(&self, role: &str, explicit_consent: bool) -> Result<Invite, Self::Error> {
        self.inner
            .create(role, explicit_consent)
            .map_err(Into::into)
    }

    fn find_by_id(&self, invite_id: &str) -> Result<Option<Invite>, Self::Error> {
        self.inner.find_by_id(invite_id).map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<Invite>, Self::Error> {
        self.inner.find_all().map_err(Into::into)
    }

    fn invalidate(&self, invite_id: &str) -> Result<(), Self::Error> {
        self.inner.invalidate(invite_id).map_err(Into::into)
    }

    fn delete(&self, invite_id: &str) -> Result<(), Self::Error> {
        self.inner.delete(invite_id).map_err(Into::into)
    }
}

// ── FavoritePodcastEpisode ──────────────────────────────────────────────────

use crate::favorite_podcast_episode::DieselFavoritePodcastEpisodeRepository;
use podfetch_domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};

pub struct FavoritePodcastEpisodeRepositoryImpl {
    inner: DieselFavoritePodcastEpisodeRepository,
}

impl FavoritePodcastEpisodeRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselFavoritePodcastEpisodeRepository::new(database),
        }
    }
}

impl FavoritePodcastEpisodeRepository for FavoritePodcastEpisodeRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id_and_episode_id(
        &self,
        user_id: i32,
        episode_id: i32,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error> {
        self.inner
            .get_by_user_id_and_episode_id(user_id, episode_id)
            .map_err(Into::into)
    }

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error> {
        self.inner.save_or_update(favorite).map_err(Into::into)
    }

    fn is_liked_by_someone(&self, episode_id: i32) -> Result<bool, Self::Error> {
        self.inner
            .is_liked_by_someone(episode_id)
            .map_err(Into::into)
    }

    fn get_favorites_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error> {
        self.inner
            .get_favorites_by_user_id(user_id)
            .map_err(Into::into)
    }
}

// ── ListeningEvent ──────────────────────────────────────────────────────────

use crate::listening_event::DieselListeningEventRepository;
use chrono::NaiveDateTime;
use podfetch_domain::listening_event::{
    ListeningEvent, ListeningEventRepository, NewListeningEvent,
};

pub struct ListeningEventRepositoryImpl {
    inner: DieselListeningEventRepository,
}

impl ListeningEventRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselListeningEventRepository::new(database),
        }
    }
}

impl ListeningEventRepository for ListeningEventRepositoryImpl {
    type Error = CustomError;

    fn create(&self, event: NewListeningEvent) -> Result<ListeningEvent, Self::Error> {
        self.inner.create(event).map_err(Into::into)
    }

    fn get_by_user_and_range(
        &self,
        user_id: i32,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, Self::Error> {
        self.inner
            .get_by_user_and_range(user_id, from, to)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: i32) -> Result<usize, Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<usize, Self::Error> {
        self.inner
            .delete_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }
}

// ── Notification ────────────────────────────────────────────────────────────

use crate::notification::DieselNotificationRepository;
use podfetch_domain::notification::{Notification, NotificationRepository};

pub struct NotificationRepositoryImpl {
    inner: DieselNotificationRepository,
}

impl NotificationRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselNotificationRepository::new(database),
        }
    }
}

impl NotificationRepository for NotificationRepositoryImpl {
    type Error = CustomError;

    fn create(&self, notification: Notification) -> Result<Notification, Self::Error> {
        self.inner.create(notification).map_err(Into::into)
    }

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error> {
        self.inner.get_unread_notifications().map_err(Into::into)
    }

    fn update_status_of_notification(&self, id: i32, status: &str) -> Result<(), Self::Error> {
        self.inner
            .update_status_of_notification(id, status)
            .map_err(Into::into)
    }
}

// ── Playlist ────────────────────────────────────────────────────────────────

use crate::playlist::DieselPlaylistRepository;
use podfetch_domain::playlist::{Playlist, PlaylistItem, PlaylistRepository};

pub struct PlaylistRepositoryImpl {
    inner: DieselPlaylistRepository,
}

impl PlaylistRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPlaylistRepository::new(database),
        }
    }
}

impl PlaylistRepository for PlaylistRepositoryImpl {
    type Error = CustomError;

    fn find_by_name(&self, name: &str) -> Result<Option<Playlist>, Self::Error> {
        self.inner.find_by_name(name).map_err(Into::into)
    }

    fn insert_playlist(&self, playlist: Playlist) -> Result<Playlist, Self::Error> {
        self.inner.insert_playlist(playlist).map_err(Into::into)
    }

    fn find_by_id(&self, playlist_id: &str) -> Result<Option<Playlist>, Self::Error> {
        self.inner.find_by_id(playlist_id).map_err(Into::into)
    }

    fn find_by_user_and_id(
        &self,
        playlist_id: &str,
        user_id: i32,
    ) -> Result<Option<Playlist>, Self::Error> {
        self.inner
            .find_by_user_and_id(playlist_id, user_id)
            .map_err(Into::into)
    }

    fn list_by_user(&self, user_id: i32) -> Result<Vec<Playlist>, Self::Error> {
        self.inner.list_by_user(user_id).map_err(Into::into)
    }

    fn update_playlist_name(
        &self,
        playlist_id: &str,
        user_id: i32,
        name: &str,
    ) -> Result<usize, Self::Error> {
        self.inner
            .update_playlist_name(playlist_id, user_id, name)
            .map_err(Into::into)
    }

    fn delete_playlist(&self, playlist_id: &str, user_id: i32) -> Result<usize, Self::Error> {
        self.inner
            .delete_playlist(playlist_id, user_id)
            .map_err(Into::into)
    }

    fn insert_playlist_item(&self, item: PlaylistItem) -> Result<PlaylistItem, Self::Error> {
        self.inner.insert_playlist_item(item).map_err(Into::into)
    }

    fn list_items_by_playlist_id(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<PlaylistItem>, Self::Error> {
        self.inner
            .list_items_by_playlist_id(playlist_id)
            .map_err(Into::into)
    }

    fn delete_items_by_playlist_id(&self, playlist_id: &str) -> Result<usize, Self::Error> {
        self.inner
            .delete_items_by_playlist_id(playlist_id)
            .map_err(Into::into)
    }

    fn delete_playlist_item(
        &self,
        playlist_id: &str,
        episode_id: i32,
    ) -> Result<usize, Self::Error> {
        self.inner
            .delete_playlist_item(playlist_id, episode_id)
            .map_err(Into::into)
    }

    fn delete_items_by_episode_id(&self, episode_id: i32) -> Result<usize, Self::Error> {
        self.inner
            .delete_items_by_episode_id(episode_id)
            .map_err(Into::into)
    }
}

// ── PodcastEpisodeChapter ───────────────────────────────────────────────────

use crate::podcast_episode_chapter::DieselPodcastEpisodeChapterRepository;
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapter, PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};

pub struct PodcastEpisodeChapterRepositoryImpl {
    inner: DieselPodcastEpisodeChapterRepository,
}

impl PodcastEpisodeChapterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastEpisodeChapterRepository::new(database),
        }
    }
}

impl PodcastEpisodeChapterRepository for PodcastEpisodeChapterRepositoryImpl {
    type Error = CustomError;

    fn upsert(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error> {
        self.inner.upsert(chapter).map_err(Into::into)
    }

    fn get_by_episode_id(
        &self,
        episode_id: i32,
    ) -> Result<Vec<PodcastEpisodeChapter>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }
}

// ── PodcastSettings ─────────────────────────────────────────────────────────

use crate::podcast_settings::DieselPodcastSettingsRepository;
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};

pub struct PodcastSettingsRepositoryImpl {
    inner: DieselPodcastSettingsRepository,
}

impl PodcastSettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastSettingsRepository::new(database),
        }
    }
}

impl PodcastSettingsRepository for PodcastSettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self, podcast_id: i32) -> Result<Option<PodcastSetting>, Self::Error> {
        self.inner.get_settings(podcast_id).map_err(Into::into)
    }

    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error> {
        self.inner.upsert_settings(setting).map_err(Into::into)
    }
}

// ── Session ─────────────────────────────────────────────────────────────────

use crate::session::DieselSessionRepository;
use podfetch_domain::session::{Session, SessionRepository};

pub struct SessionRepositoryImpl {
    inner: DieselSessionRepository,
}

impl SessionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSessionRepository::new(database),
        }
    }
}

impl SessionRepository for SessionRepositoryImpl {
    type Error = CustomError;

    fn create(&self, session: Session) -> Result<Session, Self::Error> {
        self.inner.create(session).map_err(Into::into)
    }

    fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>, Self::Error> {
        self.inner
            .find_by_session_id(session_id)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: i32) -> Result<usize, Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn cleanup_expired(&self, now: NaiveDateTime) -> Result<usize, Self::Error> {
        self.inner.cleanup_expired(now).map_err(Into::into)
    }
}

// ── Settings ────────────────────────────────────────────────────────────────

use crate::settings::DieselSettingsRepository;
use podfetch_domain::settings::{Setting, SettingRepository};

pub struct SettingsRepositoryImpl {
    inner: DieselSettingsRepository,
}

impl SettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSettingsRepository::new(database),
        }
    }
}

impl SettingRepository for SettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error> {
        self.inner.get_settings().map_err(Into::into)
    }

    fn update_settings(&self, setting: Setting) -> Result<Setting, Self::Error> {
        self.inner.update_settings(setting).map_err(Into::into)
    }

    fn insert_default_settings(&self) -> Result<(), Self::Error> {
        self.inner.insert_default_settings().map_err(Into::into)
    }
}

// ── Subscription ────────────────────────────────────────────────────────────

use crate::subscription::DieselSubscriptionRepository;
use podfetch_domain::subscription::{
    GPodderAvailablePodcast, SubscriptionModelChanges, SubscriptionRepository,
};

pub struct SubscriptionRepositoryImpl {
    inner: DieselSubscriptionRepository,
}

impl SubscriptionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSubscriptionRepository::new(database),
        }
    }
}

impl SubscriptionRepository for SubscriptionRepositoryImpl {
    type Error = CustomError;

    fn delete_by_user_id(&self, user_id: i32) -> Result<(), Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        user_id: i32,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_device_subscriptions(device_id, user_id, since, timestamp)
            .map_err(Into::into)
    }

    fn get_user_subscriptions(
        &self,
        user_id: i32,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_user_subscriptions(user_id, since, timestamp)
            .map_err(Into::into)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        user_id: i32,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.inner
            .update_subscriptions(device_id, user_id, add, remove)
            .map_err(Into::into)
    }

    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        self.inner
            .get_available_gpodder_podcasts()
            .map_err(Into::into)
    }

    fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        user_id: i32,
    ) -> Result<Vec<String>, Self::Error> {
        self.inner
            .get_active_device_podcast_urls(device_id, user_id)
            .map_err(Into::into)
    }
}

// ── Tag ─────────────────────────────────────────────────────────────────────

use crate::tag::DieselTagRepository;
use podfetch_domain::tag::{Tag, TagRepository, TagUpdate, TagsPodcast};

pub struct TagRepositoryImpl {
    inner: DieselTagRepository,
}

impl TagRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselTagRepository::new(database),
        }
    }
}

impl TagRepository for TagRepositoryImpl {
    type Error = CustomError;

    fn create(&self, tag: Tag) -> Result<Tag, Self::Error> {
        self.inner.create(tag).map_err(Into::into)
    }

    fn get_tags(&self, user_id: i32) -> Result<Vec<Tag>, Self::Error> {
        self.inner.get_tags(user_id).map_err(Into::into)
    }

    fn get_tags_of_podcast(&self, podcast_id: i32, user_id: i32) -> Result<Vec<Tag>, Self::Error> {
        self.inner
            .get_tags_of_podcast(podcast_id, user_id)
            .map_err(Into::into)
    }

    fn get_tag_by_id_and_user_id(
        &self,
        tag_id: &str,
        user_id: i32,
    ) -> Result<Option<Tag>, Self::Error> {
        self.inner
            .get_tag_by_id_and_user_id(tag_id, user_id)
            .map_err(Into::into)
    }

    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, Self::Error> {
        self.inner.update(tag_id, update).map_err(Into::into)
    }

    fn delete(&self, tag_id: &str) -> Result<(), Self::Error> {
        self.inner.delete(tag_id).map_err(Into::into)
    }

    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: i32,
    ) -> Result<TagsPodcast, Self::Error> {
        self.inner
            .add_podcast_to_tag(tag_id_to_insert, podcast_id_to_insert)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), Self::Error> {
        self.inner.delete_tag_podcasts(tag_id).map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: i32,
        tag_id: &str,
    ) -> Result<(), Self::Error> {
        self.inner
            .delete_tag_podcasts_by_podcast_id_tag_id(podcast_id, tag_id)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error> {
        self.inner
            .delete_tag_podcasts_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }
}

// ── UserAdmin ───────────────────────────────────────────────────────────────

use crate::user_admin::DieselUserAdminRepository;
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository};

pub struct UserAdminRepositoryImpl {
    inner: DieselUserAdminRepository,
}

impl UserAdminRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselUserAdminRepository::new(database),
        }
    }
}

impl UserAdminRepository for UserAdminRepositoryImpl {
    type Error = CustomError;

    fn create(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.inner.create(user).map_err(Into::into)
    }

    fn find_by_api_key(&self, api_key: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.inner.find_by_api_key(api_key).map_err(Into::into)
    }

    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.inner.find_by_username(username).map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error> {
        self.inner.find_all().map_err(Into::into)
    }

    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.inner.update(user).map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        self.inner.delete_by_username(username).map_err(Into::into)
    }
}
