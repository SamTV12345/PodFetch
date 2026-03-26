use podfetch_persistence::adapters::DeviceRepositoryImpl;
use podfetch_persistence::adapters::FavoritePodcastEpisodeRepositoryImpl;
use podfetch_persistence::adapters::FilterRepositoryImpl;
use podfetch_persistence::adapters::InviteRepositoryImpl;
use podfetch_persistence::adapters::ListeningEventRepositoryImpl;
use podfetch_persistence::adapters::NotificationRepositoryImpl;
use podfetch_persistence::adapters::PlaylistRepositoryImpl;
use podfetch_persistence::adapters::PodcastEpisodeChapterRepositoryImpl;
use podfetch_persistence::adapters::PodcastSettingsRepositoryImpl;
use podfetch_persistence::adapters::SessionRepositoryImpl;
use podfetch_persistence::adapters::SettingsRepositoryImpl;
use podfetch_persistence::adapters::SubscriptionRepositoryImpl;
use podfetch_persistence::adapters::TagRepositoryImpl;
use podfetch_persistence::adapters::UserAdminRepositoryImpl;
use crate::services::device::service::DeviceService;
use crate::services::favorite_podcast_episode::service::FavoritePodcastEpisodeService;
use crate::services::filter::service::FilterService;
use crate::services::invite::service::InviteService;
use crate::services::login::service::LoginService;
use crate::services::notification::service::NotificationService;
use crate::services::playlist::service::PlaylistService;
use crate::services::podcast_episode_chapter::service::PodcastEpisodeChapterService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::services::session::service::SessionService;
use crate::services::stats::service::StatsService;
use crate::services::subscription::service::SubscriptionService;
use crate::services::tag::service::TagService;
use crate::services::user_auth::service::UserAuthService;
use crate::services::user_admin::service::UserAdminService;
use crate::services::user_onboarding::service::UserOnboardingService;
use crate::usecases::watchtime::WatchtimeUseCase;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub device_service: Arc<DeviceService>,
    pub environment: Arc<EnvironmentService>,
    pub favorite_podcast_episode_service: Arc<FavoritePodcastEpisodeService>,
    pub filter_service: Arc<FilterService>,
    pub invite_service: Arc<InviteService>,
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
        let database = podfetch_persistence::db::database();
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
        let stats_service = Arc::new(StatsService::new(Arc::new(
            crate::services::listening_event::service::ListeningEventService::new(
                Arc::new(ListeningEventRepositoryImpl::new(database.clone())),
            ),
        )));
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
