#[derive(Clone, Debug, Default)]
pub struct Podcast {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub directory_id: String,
    pub(crate) rssfeed: String,
    pub image_url: String,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<String>,
    pub keywords: Option<String>,
    pub last_build_date: Option<String>,
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    pub directory_name: String,
}