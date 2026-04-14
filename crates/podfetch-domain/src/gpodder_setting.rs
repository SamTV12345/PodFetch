pub struct GpodderSetting {
    pub id: i32,
    pub user_id: i32,
    pub scope: String,
    pub scope_id: Option<String>,
    pub data: String, // JSON string
}

pub trait GpodderSettingRepository: Send + Sync {
    type Error;

    fn get_setting(
        &self,
        user_id: i32,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error>;
    fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, Self::Error>;
}
