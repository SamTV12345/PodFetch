use crate::adapters::persistence::repositories::device::device_repository::DeviceRepositoryImpl;
use crate::adapters::persistence::repositories::favorite_podcast_episode_repository::FavoritePodcastEpisodeRepositoryImpl;
use crate::adapters::persistence::repositories::filter_repository::FilterRepositoryImpl;
use crate::adapters::persistence::repositories::invite_repository::InviteRepositoryImpl;
use crate::adapters::persistence::repositories::listening_event_repository::ListeningEventRepositoryImpl;
use crate::adapters::persistence::repositories::notification_repository::NotificationRepositoryImpl;
use crate::adapters::persistence::repositories::playlist_repository::PlaylistRepositoryImpl;
use crate::adapters::persistence::repositories::podcast_episode_chapter_repository::PodcastEpisodeChapterRepositoryImpl;
use crate::adapters::persistence::repositories::podcast_settings_repository::PodcastSettingsRepositoryImpl;
use crate::adapters::persistence::repositories::session_repository::SessionRepositoryImpl;
use crate::adapters::persistence::repositories::settings_repository::SettingsRepositoryImpl;
use crate::adapters::persistence::repositories::subscription_repository::SubscriptionRepositoryImpl;
use crate::adapters::persistence::repositories::tag_repository::TagRepositoryImpl;
use crate::adapters::persistence::repositories::user_admin_repository::UserAdminRepositoryImpl;
use crate::application::services::device::service::DeviceService;
use crate::application::services::favorite_podcast_episode::service::FavoritePodcastEpisodeService;
use crate::application::services::filter::service::FilterService;
use crate::application::services::invite::service::InviteService;
use crate::application::services::login::service::LoginService;
use crate::application::services::notification::service::NotificationService;
use crate::application::services::playlist::service::PlaylistService;
use crate::application::services::podcast_episode_chapter::service::PodcastEpisodeChapterService;
use crate::application::services::podcast_settings::service::PodcastSettingsService;
use crate::application::services::settings::service::SettingsService;
use crate::application::services::listening_event::service::ListeningEventService;
use crate::application::services::session::service::SessionService;
use crate::application::services::stats::service::StatsService;
use crate::application::services::subscription::service::SubscriptionService;
use crate::application::services::tag::service::TagService;
use crate::application::services::user_auth::service::UserAuthService;
use crate::application::services::user_admin::service::UserAdminService;
use crate::application::services::user_onboarding::service::UserOnboardingService;
use crate::application::usecases::watchtime::WatchtimeUseCase;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use common_infrastructure::config::EnvironmentService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub device_service: Arc<DeviceService>,
    pub environment: Arc<EnvironmentService>,
    pub favorite_podcast_episode_service: Arc<FavoritePodcastEpisodeService>,
    pub filter_service: Arc<FilterService>,
    pub invite_service: Arc<InviteService>,
    pub listening_event_service: Arc<ListeningEventService>,
    pub login_service: Arc<LoginService>,
    pub notification_service: Arc<NotificationService>,
    pub playlist_service: Arc<PlaylistService>,
    pub podcast_episode_chapter_service: Arc<PodcastEpisodeChapterService>,
    pub podcast_settings_service: Arc<PodcastSettingsService>,
    pub session_service: Arc<SessionService>,
    pub settings_service: Arc<SettingsService>,
    pub stats_service: Arc<StatsService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub tag_service: Arc<TagService>,
    pub user_admin_service: Arc<UserAdminService>,
    pub user_auth_service: Arc<UserAuthService>,
    pub user_onboarding_service: Arc<UserOnboardingService>,
    pub watchtime_service: Arc<WatchtimeUseCase>,
}

impl AppState {
    pub fn new() -> Self {
        let database = crate::adapters::persistence::dbconfig::db::database();
        let environment = Arc::new(ENVIRONMENT_SERVICE.clone());
        let device_service = Arc::new(DeviceService::new(Arc::new(DeviceRepositoryImpl::new(
            database.clone(),
        ))));
        let favorite_podcast_episode_service = Arc::new(FavoritePodcastEpisodeService::new(
            Arc::new(FavoritePodcastEpisodeRepositoryImpl::new(database.clone())),
        ));
        let filter_service = Arc::new(FilterService::new(Arc::new(FilterRepositoryImpl::new(
            database.clone(),
        ))));
        let invite_service = Arc::new(InviteService::new(
            Arc::new(InviteRepositoryImpl::new(database.clone())),
            environment.clone(),
        ));
        let listening_event_service = Arc::new(ListeningEventService::new(Arc::new(
            ListeningEventRepositoryImpl::new(database.clone()),
        )));
        let user_auth_service = Arc::new(UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
            environment.clone(),
        ));
        let login_service = Arc::new(LoginService::new(
            environment.clone(),
            user_auth_service.clone(),
        ));
        let notification_service = Arc::new(NotificationService::new(Arc::new(
            NotificationRepositoryImpl::new(database.clone()),
        )));
        let playlist_service = Arc::new(PlaylistService::new(Arc::new(
            PlaylistRepositoryImpl::new(database.clone()),
        )));
        let podcast_episode_chapter_service = Arc::new(PodcastEpisodeChapterService::new(
            Arc::new(PodcastEpisodeChapterRepositoryImpl::new(database.clone())),
        ));
        let podcast_settings_service = Arc::new(PodcastSettingsService::new(Arc::new(
            PodcastSettingsRepositoryImpl::new(database.clone()),
        )));
        let session_service = Arc::new(SessionService::new(Arc::new(SessionRepositoryImpl::new(
            database.clone(),
        ))));
        let settings_service = Arc::new(SettingsService::new(Arc::new(
            SettingsRepositoryImpl::new(database.clone()),
        )));
        let stats_service = Arc::new(StatsService::new(listening_event_service.clone()));
        let subscription_service = Arc::new(SubscriptionService::new(Arc::new(
            SubscriptionRepositoryImpl::new(database.clone()),
        )));
        let tag_service = Arc::new(TagService::new(Arc::new(TagRepositoryImpl::new(
            database.clone(),
        ))));
        let watchtime_service = Arc::new(WatchtimeUseCase::new());
        let user_admin_service = Arc::new(UserAdminService::new(
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
            environment.clone(),
        ));
        let user_onboarding_service = Arc::new(UserOnboardingService::new(
            invite_service.clone(),
            Arc::new(UserAdminRepositoryImpl::new(database)),
        ));

        Self {
            device_service,
            environment,
            favorite_podcast_episode_service,
            filter_service,
            invite_service,
            listening_event_service,
            login_service,
            notification_service,
            playlist_service,
            podcast_episode_chapter_service,
            podcast_settings_service,
            session_service,
            settings_service,
            stats_service,
            subscription_service,
            tag_service,
            user_admin_service,
            user_auth_service,
            user_onboarding_service,
            watchtime_service,
        }
    }
}
