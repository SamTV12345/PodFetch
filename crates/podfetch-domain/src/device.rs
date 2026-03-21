#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub id: Option<i32>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub username: String,
}

pub trait DeviceRepository: Send + Sync {
    type Error;

    fn create(&self, device: Device) -> Result<Device, Self::Error>;
    fn get_devices_of_user(&self, username: &str) -> Result<Vec<Device>, Self::Error>;
    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;
}
