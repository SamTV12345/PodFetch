use crate::adapters::persistence::repositories::device::device_repository::DeviceRepositoryImpl;
use crate::adapters::persistence::repositories::invite_repository::InviteRepositoryImpl;
use crate::adapters::persistence::repositories::settings_repository::SettingsRepositoryImpl;
use crate::adapters::persistence::repositories::tag_repository::DieselTagRepository;
use crate::adapters::persistence::repositories::user_admin_repository::UserAdminRepositoryImpl;
use crate::application::services::device::service::DeviceService;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::service::environment_service::EnvironmentService;
use crate::service::invite_service::InviteService;
use crate::service::login_service::LoginService;
use crate::service::settings_service::SettingsService;
use crate::service::tag_service::TagService;
use crate::service::user_admin_service::UserAdminService;
use crate::service::user_onboarding_service::UserOnboardingService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub device_service: Arc<DeviceService>,
    pub environment: Arc<EnvironmentService>,
    pub invite_service: Arc<InviteService>,
    pub login_service: Arc<LoginService>,
    pub settings_service: Arc<SettingsService>,
    pub tag_service: Arc<TagService>,
    pub user_admin_service: Arc<UserAdminService>,
    pub user_onboarding_service: Arc<UserOnboardingService>,
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
        let login_service = Arc::new(LoginService::new(environment.clone()));
        let settings_service = Arc::new(SettingsService::new(Arc::new(
            SettingsRepositoryImpl::new(database.clone()),
        )));
        let tag_service = Arc::new(TagService::new(Arc::new(DieselTagRepository::new(
            database.clone(),
        ))));
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
            settings_service,
            tag_service,
            user_admin_service,
            user_onboarding_service,
        }
    }
}
