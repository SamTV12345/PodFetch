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

    pub fn get_by_username(&self, username: &str) -> Result<Vec<DeviceSyncGroup>, CustomError> {
        self.repository.get_by_username(username)
    }

    pub fn replace_all(
        &self,
        username: &str,
        groups: Vec<DeviceSyncGroup>,
    ) -> Result<(), CustomError> {
        self.repository.replace_all(username, groups)
    }
}
