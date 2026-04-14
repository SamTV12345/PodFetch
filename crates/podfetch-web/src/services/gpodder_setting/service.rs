use common_infrastructure::error::CustomError;
use podfetch_domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct GpodderSettingService {
    repository: Arc<dyn GpodderSettingRepository<Error = CustomError>>,
}

impl GpodderSettingService {
    pub fn new(repository: Arc<dyn GpodderSettingRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn get_setting(
        &self,
        user_id: i32,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, CustomError> {
        self.repository.get_setting(user_id, scope, scope_id)
    }

    pub fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, CustomError> {
        self.repository.save_setting(setting)
    }
}
