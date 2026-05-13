//! Audiobookshelf-compatible IDs for library items and episodes.
//!
//! Audiobookshelf models everything as a `library_item_id` (book or podcast),
//! optionally paired with an `episode_id` (for podcasts) or a chapter index (for
//! books). PodFetch reuses its existing integer primary keys, so we encode them
//! with a `li_pod_<id>` / `li_book_<id>` prefix and `ep_<id>` for episodes.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryItemId {
    Podcast(i32),
    Book(String),
}

impl LibraryItemId {
    pub fn podcast(id: i32) -> Self {
        Self::Podcast(id)
    }

    pub fn book(uuid: impl Into<String>) -> Self {
        Self::Book(uuid.into())
    }

    pub fn parse(value: &str) -> Option<Self> {
        if let Some(rest) = value.strip_prefix("li_pod_") {
            rest.parse::<i32>().ok().map(Self::Podcast)
        } else if let Some(rest) = value.strip_prefix("li_book_") {
            Some(Self::Book(rest.to_string()))
        } else {
            None
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
pub struct EpisodeId(pub i32);

impl EpisodeId {
    pub fn parse(value: &str) -> Option<Self> {
        value.strip_prefix("ep_")?.parse::<i32>().ok().map(Self)
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
        let id = LibraryItemId::Podcast(42);
        let s = id.as_string();
        assert_eq!(s, "li_pod_42");
        assert_eq!(LibraryItemId::parse(&s), Some(LibraryItemId::Podcast(42)));
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
        let id = EpisodeId(7);
        let s = id.as_string();
        assert_eq!(s, "ep_7");
        assert_eq!(EpisodeId::parse(&s), Some(EpisodeId(7)));
    }
}
