pub struct DeviceSyncGroup {
    pub id: i32,
    pub username: String,
    pub group_id: i32,
    pub device_id: String,
}

pub trait DeviceSyncGroupRepository: Send + Sync {
    type Error;

    fn get_by_username(&self, username: &str) -> Result<Vec<DeviceSyncGroup>, Self::Error>;
    fn replace_all(&self, username: &str, groups: Vec<DeviceSyncGroup>) -> Result<(), Self::Error>;
}
