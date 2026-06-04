use uuid::Uuid;

pub struct DeviceSyncGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: i32,
    pub device_id: String,
}

pub trait DeviceSyncGroupRepository: Send + Sync {
    type Error;

    fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<DeviceSyncGroup>, Self::Error>;
    fn replace_all(&self, user_id: Uuid, groups: Vec<DeviceSyncGroup>) -> Result<(), Self::Error>;
}
