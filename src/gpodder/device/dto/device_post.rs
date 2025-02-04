use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct DevicePost {
    pub caption: String,
    #[serde(rename = "type")]
    pub kind: String,
}
