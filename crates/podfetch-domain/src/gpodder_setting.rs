pub struct GpodderSetting {
    pub id: i32,
    pub username: String,
    pub scope: String,
    pub scope_id: Option<String>,
    pub data: String, // JSON string
}

pub trait GpodderSettingRepository: Send + Sync {
    type Error;

    fn get_setting(
        &self,
        username: &str,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error>;
    fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, Self::Error>;
}
