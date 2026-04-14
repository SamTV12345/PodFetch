pub struct DeviceSyncGroup {
    pub id: i32,
    pub user_id: i32,
    pub group_id: i32,
    pub device_id: String,
}

pub trait DeviceSyncGroupRepository: Send + Sync {
    type Error;

    fn get_by_user_id(&self, user_id: i32) -> Result<Vec<DeviceSyncGroup>, Self::Error>;
    fn replace_all(&self, user_id: i32, groups: Vec<DeviceSyncGroup>) -> Result<(), Self::Error>;
}
