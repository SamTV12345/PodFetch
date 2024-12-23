#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterDto {
    pub username: String,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}