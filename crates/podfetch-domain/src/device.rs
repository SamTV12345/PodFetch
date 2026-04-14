#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub id: Option<i32>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub user_id: i32,
}

pub trait DeviceRepository: Send + Sync {
    type Error;

    fn create(&self, device: Device) -> Result<Device, Self::Error>;
    fn get_devices_of_user(&self, user_id: i32) -> Result<Vec<Device>, Self::Error>;
    fn delete_by_user_id(&self, user_id: i32) -> Result<(), Self::Error>;
}
