#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RSSAPiKey {
    pub api_key: String,
}