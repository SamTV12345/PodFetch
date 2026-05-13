//! Resolves a book's metadata from a fixed precedence chain.
//!
//! For Phase B v1 the chain is `folderStructure → audioMetatags`. Later phases
//! will extend this to nfoFile / txtFiles / opfFile / absMetadata.

use crate::services::audiobookshelf::audio_probe::ProbedTags;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ResolvedBookMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_year: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub language: Option<String>,
    pub explicit: bool,
    pub authors: Vec<String>,
    pub narrators: Vec<String>,
    pub series: Option<(String, Option<String>)>,
}

pub fn resolve(folder: &Path, tags: &ProbedTags) -> ResolvedBookMetadata {
    let mut meta = ResolvedBookMetadata::default();
    apply_folder_structure(&mut meta, folder);
    apply_audio_tags(&mut meta, tags);
    if meta.title.is_empty() {
        meta.title = "Unknown Title".to_string();
    }
    meta
}

/// Parse paths like `<library>/<Author>/<Title>` or `<library>/<Author>/<Series #N>/<Title>`.
/// The last folder segment is the book title; the one above (when present) is
/// the author. A middle segment of the form "<Series> #<sequence>" is treated
/// as a series.
fn apply_folder_structure(meta: &mut ResolvedBookMetadata, folder: &Path) {
    let mut segments: Vec<&str> = folder
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let Some(title_seg) = segments.pop() else {
        return;
    };
    meta.title = title_seg.to_string();
    let Some(parent_seg) = segments.pop() else {
        return;
    };
    if let Some((series_name, sequence)) = parse_series_segment(parent_seg) {
        meta.series = Some((series_name, sequence));
        if let Some(author_seg) = segments.pop() {
            meta.authors.push(author_seg.to_string());
        }
    } else {
        meta.authors.push(parent_seg.to_string());
    }
}

fn parse_series_segment(segment: &str) -> Option<(String, Option<String>)> {
    let trimmed = segment.trim();
    if let Some(hash_idx) = trimmed.rfind(" #") {
        let name = trimmed[..hash_idx].trim();
        let seq = trimmed[hash_idx + 2..].trim();
        if !name.is_empty() && !seq.is_empty() {
            return Some((name.to_string(), Some(seq.to_string())));
        }
    }
    None
}

fn apply_audio_tags(meta: &mut ResolvedBookMetadata, tags: &ProbedTags) {
    if let Some(value) = tags.album.clone().or_else(|| tags.title.clone())
        && !value.trim().is_empty()
    {
        meta.title = value;
    }
    if let Some(artist) = tags.album_artist.clone().or_else(|| tags.artist.clone()) {
        let authors = split_multi(&artist);
        if !authors.is_empty() {
            meta.authors = authors;
        }
    }
    if let Some(composer) = tags.composer.clone() {
        let narrators = split_multi(&composer);
        if !narrators.is_empty() {
            meta.narrators = narrators;
        }
    }
    meta.description = meta
        .description
        .clone()
        .or_else(|| tags.description.clone().or_else(|| tags.comment.clone()));
    meta.publisher = meta.publisher.clone().or_else(|| tags.publisher.clone());
    meta.language = meta.language.clone().or_else(|| tags.language.clone());
    meta.isbn = meta.isbn.clone().or_else(|| tags.isbn.clone());
    meta.asin = meta.asin.clone().or_else(|| tags.asin.clone());
    meta.published_year = meta
        .published_year
        .clone()
        .or_else(|| tags.year.clone())
        .or_else(|| {
            tags.date
                .as_deref()
                .map(|d| d.chars().take(4).collect::<String>())
                .filter(|s| s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty())
        });
    meta.published_date = meta.published_date.clone().or_else(|| tags.date.clone());
    if let Some(name) = tags.series.clone().or_else(|| tags.grouping.clone()) {
        let sequence = tags.series_part.clone();
        meta.series = Some((name, sequence));
    }
    meta.explicit = meta.explicit
        || matches!(
            tags.explicit.as_deref(),
            Some("1") | Some("yes") | Some("true") | Some("explicit") | Some("4")
        );
}

fn split_multi(input: &str) -> Vec<String> {
    input
        .split([',', ';', '&'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_structure_with_author_and_title() {
        let mut tags = ProbedTags::default();
        tags.title = Some("Embedded Title".to_string());
        let meta = resolve(
            Path::new("/audiobooks/Andy Weir/Project Hail Mary"),
            &ProbedTags::default(),
        );
        assert_eq!(meta.title, "Project Hail Mary");
        assert_eq!(meta.authors, vec!["Andy Weir".to_string()]);
        // Audio tags override title
        let meta2 = resolve(
            Path::new("/audiobooks/Andy Weir/Project Hail Mary"),
            &tags,
        );
        assert_eq!(meta2.title, "Embedded Title");
    }

    #[test]
    fn folder_structure_with_series_segment() {
        let meta = resolve(
            Path::new("/audiobooks/Brandon Sanderson/Mistborn #2/The Well of Ascension"),
            &ProbedTags::default(),
        );
        assert_eq!(meta.title, "The Well of Ascension");
        assert_eq!(meta.authors, vec!["Brandon Sanderson".to_string()]);
        assert_eq!(
            meta.series.as_ref().map(|(s, n)| (s.clone(), n.clone())),
            Some(("Mistborn".to_string(), Some("2".to_string())))
        );
    }

    #[test]
    fn audio_tags_override_when_present() {
        let mut tags = ProbedTags::default();
        tags.album = Some("Tag Album".to_string());
        tags.album_artist = Some("Tag Artist".to_string());
        tags.composer = Some("Reader A, Reader B".to_string());
        tags.year = Some("2024".to_string());
        tags.series = Some("Tag Series".to_string());
        tags.series_part = Some("3".to_string());
        let meta = resolve(Path::new("/audiobooks/Folder Author/Folder Title"), &tags);
        assert_eq!(meta.title, "Tag Album");
        assert_eq!(meta.authors, vec!["Tag Artist".to_string()]);
        assert_eq!(meta.narrators, vec!["Reader A".to_string(), "Reader B".to_string()]);
        assert_eq!(meta.published_year.as_deref(), Some("2024"));
        assert_eq!(
            meta.series.as_ref().map(|(s, n)| (s.clone(), n.clone())),
            Some(("Tag Series".to_string(), Some("3".to_string())))
        );
    }
}
