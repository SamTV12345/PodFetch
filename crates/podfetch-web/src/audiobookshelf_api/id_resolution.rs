//! Backwards-compatible parsing of incoming audiobookshelf (ABS) ids.
//!
//! The UUID migration re-typed PodFetch's primary keys, but ABS mobile apps
//! cache the *opaque* id strings they were handed before the migration —
//! `li_pod_{int}`, `pod_{int}`, `ep_{int}`, `ino_ep_{int}`. Those legacy ids
//! embed the pre-migration integer pk, not the new UUID.
//!
//! The domain `LibraryItemId::parse` / `EpisodeId::parse` are pure (no DB), so
//! they can only validate the UUID form. Legacy resolution needs a DB lookup
//! (`find_by_legacy_id`), so it lives here in the web layer where the services
//! are reachable.
//!
//! These helpers strip the ABS prefix, run the shared
//! [`parse_resolved_id`](crate::controllers::id_resolver::parse_resolved_id)
//! (UUID first, integer fallback), and resolve legacy integers to the
//! current entity via the same service methods the internal API uses. They
//! return the *current* UUID so every downstream lookup and every OUTGOING id
//! stays UUID-shaped.

use crate::controllers::id_resolver::{ResolvedId, parse_resolved_id};
use crate::services::podcast::service::PodcastService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use uuid::Uuid;

/// Resolve a podcast suffix (the part after a `li_pod_` / `pod_` / `ino_pod_`
/// prefix has been stripped) to the podcast's current UUID, accepting both the
/// new UUID form and the legacy integer form.
///
/// - UUID  → confirmed via [`PodcastService::get_podcast`].
/// - i64   → resolved via [`PodcastService::get_podcast_by_legacy_id`].
fn resolve_podcast_suffix(suffix: &str) -> Result<Uuid, CustomError> {
    match parse_resolved_id(suffix)? {
        ResolvedId::Uuid(uuid) => {
            // Confirm it exists so a bogus-but-well-formed UUID still 404s,
            // matching the pre-change behaviour of `get_podcast`.
            let _ = PodcastService::get_podcast(uuid)?;
            Ok(uuid)
        }
        ResolvedId::Legacy(legacy) => {
            let podcast = PodcastService::get_podcast_by_legacy_id(legacy)?;
            Uuid::parse_str(&podcast.id)
                .map_err(|_| CustomError::from(CustomErrorInner::NotFound(Debug)))
        }
    }
}

/// Resolve an episode suffix (the part after an `ep_` / `ino_ep_` prefix has
/// been stripped) to the episode's current UUID, accepting both the new UUID
/// form and the legacy integer form.
///
/// - UUID  → returned as-is (existence is verified at the call site against the
///   podcast's episode list, mirroring the pre-change flow).
/// - i64   → resolved via [`PodcastEpisodeService::get_podcast_episode_by_legacy_id`].
fn resolve_episode_suffix(suffix: &str) -> Result<Uuid, CustomError> {
    match parse_resolved_id(suffix)? {
        ResolvedId::Uuid(uuid) => Ok(uuid),
        ResolvedId::Legacy(legacy) => {
            let episode = PodcastEpisodeService::get_podcast_episode_by_legacy_id(legacy)?
                .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
            Uuid::parse_str(&episode.id)
                .map_err(|_| CustomError::from(CustomErrorInner::NotFound(Debug)))
        }
    }
}

/// Resolve a library-item id of the podcast form (`li_pod_{uuid|int}`) to the
/// podcast's current UUID. Rejects `li_book_…` and anything non-podcast with a
/// `NotFound` so callers can keep their book branch separate.
pub fn resolve_podcast_library_item(library_item_id: &str) -> Result<Uuid, CustomError> {
    let suffix = library_item_id
        .strip_prefix("li_pod_")
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    resolve_podcast_suffix(suffix)
}

/// Resolve an `ep_{uuid|int}` episode id to the episode's current UUID.
pub fn resolve_episode(episode_id: &str) -> Result<Uuid, CustomError> {
    let suffix = episode_id
        .strip_prefix("ep_")
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    resolve_episode_suffix(suffix)
}

/// Resolve an `ino_ep_{uuid|int}` audio-file ino to the episode's current
/// UUID.
pub fn resolve_episode_ino(ino: &str) -> Result<Uuid, CustomError> {
    let suffix = ino
        .strip_prefix("ino_ep_")
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Debug)))?;
    resolve_episode_suffix(suffix)
}

/// Classifies an incoming library-item id by ABS media type *without* touching
/// the DB. Used where the only decision is podcast-vs-book routing (e.g.
/// progress-row `mediaType`). Accepts both the legacy integer and the UUID
/// form for podcasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryItemKind {
    Podcast,
    Book,
    Unknown,
}

impl LibraryItemKind {
    pub fn classify(library_item_id: &str) -> Self {
        if library_item_id.starts_with("li_pod_") {
            Self::Podcast
        } else if library_item_id.starts_with("li_book_") {
            Self::Book
        } else {
            Self::Unknown
        }
    }

    pub fn media_type_str(self) -> &'static str {
        match self {
            // Unknown ids historically defaulted to "podcast" in the progress
            // upsert path; preserve that to avoid changing stored rows.
            Self::Podcast | Self::Unknown => "podcast",
            Self::Book => "book",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_distinguishes_podcast_book_and_unknown() {
        assert_eq!(
            LibraryItemKind::classify("li_pod_42"),
            LibraryItemKind::Podcast
        );
        assert_eq!(
            LibraryItemKind::classify(&format!("li_pod_{}", Uuid::nil())),
            LibraryItemKind::Podcast
        );
        assert_eq!(
            LibraryItemKind::classify("li_book_abc-123"),
            LibraryItemKind::Book
        );
        assert_eq!(LibraryItemKind::classify("foo_1"), LibraryItemKind::Unknown);
    }

    #[test]
    fn media_type_str_maps_kind() {
        assert_eq!(LibraryItemKind::Podcast.media_type_str(), "podcast");
        assert_eq!(LibraryItemKind::Book.media_type_str(), "book");
        // Unknown preserves the historical podcast default.
        assert_eq!(LibraryItemKind::Unknown.media_type_str(), "podcast");
    }

    #[test]
    fn resolvers_reject_wrong_prefixes_before_any_db_lookup() {
        // The prefix mismatch short-circuits with NotFound *before* touching
        // the DB, so this is safe to assert without a database.
        assert!(resolve_podcast_library_item("li_book_abc").is_err());
        assert!(resolve_podcast_library_item("pod_42").is_err());
        assert!(resolve_episode("ino_ep_7").is_err());
        assert!(resolve_episode("li_pod_42").is_err());
        assert!(resolve_episode_ino("ep_7").is_err());
    }
}
