#[derive(Serialize, Deserialize, Clone)]
pub struct DevicePost {
    pub caption: String,
    #[serde(rename = "type")]
    pub kind: String,
}
