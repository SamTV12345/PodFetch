use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Podcast,
    Book,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Podcast => "podcast",
            MediaType::Book => "book",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "podcast" => Some(MediaType::Podcast),
            "book" => Some(MediaType::Book),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub media_type: MediaType,
    pub icon: String,
    pub display_order: i32,
    pub folder_paths: Vec<String>,
    pub metadata_precedence: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub trait LibraryRepository: Send + Sync {
    type Error;

    fn list(&self) -> Result<Vec<Library>, Self::Error>;
    fn find_by_id(&self, id: &str) -> Result<Option<Library>, Self::Error>;
    fn find_first_by_media_type(
        &self,
        media_type: &MediaType,
    ) -> Result<Option<Library>, Self::Error>;
    fn upsert(&self, library: Library) -> Result<Library, Self::Error>;
    fn delete(&self, id: &str) -> Result<usize, Self::Error>;
}
