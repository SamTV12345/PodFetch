#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_init: bool,
    pub language: String,
}

impl StatusResponse {
    pub fn new() -> StatusResponse {
        StatusResponse {
            is_init: true,
            language: "en-us".to_string(),
        }
    }
}