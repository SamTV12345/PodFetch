use common_infrastructure::error::CustomError;
use podfetch_domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct DeviceSyncGroupService {
    repository: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>,
}

impl DeviceSyncGroupService {
    pub fn new(repository: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn get_by_user_id(&self, user_id: i32) -> Result<Vec<DeviceSyncGroup>, CustomError> {
        self.repository.get_by_user_id(user_id)
    }

    pub fn replace_all(
        &self,
        user_id: i32,
        groups: Vec<DeviceSyncGroup>,
    ) -> Result<(), CustomError> {
        self.repository.replace_all(user_id, groups)
    }
}
