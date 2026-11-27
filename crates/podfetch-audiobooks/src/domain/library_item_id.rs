//! Audiobookshelf-compatible IDs for library items and episodes.
//!
//! Audiobookshelf models everything as a `library_item_id` (book or podcast),
//! optionally paired with an `episode_id` (for podcasts) or a chapter index (for
//! books). PodFetch reuses its existing primary keys, so we encode them
//! with a `li_pod_<id>` / `li_book_<id>` prefix and `ep_<id>` for episodes.

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryItemId {
    Podcast(Uuid),
    Book(String),
}

impl LibraryItemId {
    pub fn podcast(id: Uuid) -> Self {
        Self::Podcast(id)
    }

    pub fn book(uuid: impl Into<String>) -> Self {
        Self::Book(uuid.into())
    }

    pub fn parse(value: &str) -> Option<Self> {
        if let Some(rest) = value.strip_prefix("li_pod_") {
            Uuid::parse_str(rest).ok().map(Self::Podcast)
        } else {
            value
                .strip_prefix("li_book_")
                .map(|rest| Self::Book(rest.to_string()))
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::Podcast(id) => format!("li_pod_{id}"),
            Self::Book(uuid) => format!("li_book_{uuid}"),
        }
    }

    pub fn media_type_str(&self) -> &'static str {
        match self {
            Self::Podcast(_) => "podcast",
            Self::Book(_) => "book",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeId(pub Uuid);

impl EpisodeId {
    pub fn parse(value: &str) -> Option<Self> {
        Uuid::parse_str(value.strip_prefix("ep_")?).ok().map(Self)
    }

    pub fn as_string(&self) -> String {
        format!("ep_{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_podcast_id() {
        let uuid = Uuid::new_v4();
        let id = LibraryItemId::Podcast(uuid);
        let s = id.as_string();
        assert_eq!(s, format!("li_pod_{uuid}"));
        assert_eq!(LibraryItemId::parse(&s), Some(LibraryItemId::Podcast(uuid)));
    }

    #[test]
    fn roundtrip_book_id() {
        let id = LibraryItemId::Book("abc-123".to_string());
        let s = id.as_string();
        assert_eq!(s, "li_book_abc-123");
        assert_eq!(
            LibraryItemId::parse(&s),
            Some(LibraryItemId::Book("abc-123".to_string()))
        );
    }

    #[test]
    fn rejects_unknown_prefix() {
        assert_eq!(LibraryItemId::parse("foo_1"), None);
    }

    #[test]
    fn episode_id_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = EpisodeId(uuid);
        let s = id.as_string();
        assert_eq!(s, format!("ep_{uuid}"));
        assert_eq!(EpisodeId::parse(&s), Some(EpisodeId(uuid)));
    }
}
