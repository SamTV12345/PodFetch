use diesel::{RunQueryDsl};

// only for http
pub struct DeviceSubscription {
    pub id: String,
    pub caption: String,
    pub r#type: DeviceType,
    pub subscriptions: u32,
}

// only for http
pub struct DeviceUpdateEvent{
    pub caption: Option<String>,
    pub r#type: Option<DeviceType>
}

pub enum DeviceType {
    Mobile,
    Desktop,
    Laptop,
    Server,
    Other,
    Web,
    Unknown,
}

impl DeviceType {
    pub fn from_string(s: &str) -> DeviceType {
        match s {
            "mobile" => DeviceType::Mobile,
            "desktop" => DeviceType::Desktop,
            "laptop" => DeviceType::Laptop,
            "server" => DeviceType::Server,
            "other" => DeviceType::Other,
            "web" => DeviceType::Web,
            _ => DeviceType::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DeviceType::Mobile => "mobile".to_string(),
            DeviceType::Desktop => "desktop".to_string(),
            DeviceType::Laptop => "laptop".to_string(),
            DeviceType::Server => "server".to_string(),
            DeviceType::Other => "other".to_string(),
            DeviceType::Web => "web".to_string(),
            DeviceType::Unknown => "unknown".to_string(),
        }
    }
}