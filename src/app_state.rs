use crate::adapters::persistence::repositories::device::device_repository::DeviceRepositoryImpl;
use crate::adapters::persistence::repositories::invite_repository::InviteRepositoryImpl;
use crate::adapters::persistence::repositories::notification_repository::NotificationRepositoryImpl;
use crate::adapters::persistence::repositories::podcast_settings_repository::PodcastSettingsRepositoryImpl;
use crate::adapters::persistence::repositories::session_repository::SessionRepositoryImpl;
use crate::adapters::persistence::repositories::settings_repository::SettingsRepositoryImpl;
use crate::adapters::persistence::repositories::subscription_repository::SubscriptionRepositoryImpl;
use crate::adapters::persistence::repositories::tag_repository::TagRepositoryImpl;
use crate::adapters::persistence::repositories::user_admin_repository::UserAdminRepositoryImpl;
use crate::application::services::device::service::DeviceService;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::service::environment_service::EnvironmentService;
use crate::service::invite_service::InviteService;
use crate::service::login_service::LoginService;
use crate::service::notification_service::NotificationService;
use crate::service::playlist_service::PlaylistService;
use crate::service::podcast_settings_service::PodcastSettingsService;
use crate::service::session_service::SessionService;
use crate::service::settings_service::SettingsService;
use crate::service::stats_service::StatsService;
use crate::service::subscription::SubscriptionService;
use crate::service::tag_service::TagService;
use crate::service::user_admin_service::UserAdminService;
use crate::service::user_auth_service::UserAuthService;
use crate::service::user_onboarding_service::UserOnboardingService;
use crate::service::watchtime_service::WatchtimeService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub device_service: Arc<DeviceService>,
    pub environment: Arc<EnvironmentService>,
    pub invite_service: Arc<InviteService>,
    pub login_service: Arc<LoginService>,
    pub notification_service: Arc<NotificationService>,
    pub playlist_service: Arc<PlaylistService>,
    pub podcast_settings_service: Arc<PodcastSettingsService>,
    pub session_service: Arc<SessionService>,
    pub settings_service: Arc<SettingsService>,
    pub stats_service: Arc<StatsService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub tag_service: Arc<TagService>,
    pub user_admin_service: Arc<UserAdminService>,
    pub user_auth_service: Arc<UserAuthService>,
    pub user_onboarding_service: Arc<UserOnboardingService>,
    pub watchtime_service: Arc<WatchtimeService>,
}

impl AppState {
    pub fn new() -> Self {
        let database = crate::adapters::persistence::dbconfig::db::database();
        let environment = Arc::new(ENVIRONMENT_SERVICE.clone());
        let device_service = Arc::new(DeviceService::new(Arc::new(DeviceRepositoryImpl::new(
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
        let playlist_service = Arc::new(PlaylistService::new());
        let podcast_settings_service = Arc::new(PodcastSettingsService::new(Arc::new(
            PodcastSettingsRepositoryImpl::new(database.clone()),
        )));
        let session_service = Arc::new(SessionService::new(Arc::new(SessionRepositoryImpl::new(
            database.clone(),
        ))));
        let settings_service = Arc::new(SettingsService::new(Arc::new(
            SettingsRepositoryImpl::new(database.clone()),
        )));
        let stats_service = Arc::new(StatsService::new());
        let subscription_service = Arc::new(SubscriptionService::new(Arc::new(
            SubscriptionRepositoryImpl::new(database.clone()),
        )));
        let tag_service = Arc::new(TagService::new(Arc::new(TagRepositoryImpl::new(
            database.clone(),
        ))));
        let watchtime_service = Arc::new(WatchtimeService::new());
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
            invite_service,
            login_service,
            notification_service,
            playlist_service,
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
