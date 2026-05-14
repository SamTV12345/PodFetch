//! Walks a library's folder paths, groups files into books, probes metadata,
//! and persists to the audiobookshelf-domain tables.

use crate::services::audiobookshelf::audio_probe::{
    self, ProbedAudioFile, ProbedTags, is_supported_audio, mime_for_ext,
};
use crate::services::audiobookshelf::cover_extractor;
use crate::services::audiobookshelf::library_service::AudiobookshelfLibraryService;
use crate::services::audiobookshelf::metadata_resolver;
use chrono::Utc;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::book::{
    AuthorRepository, Book, BookAudioFile, BookAudioFileRepository, BookChapter,
    BookChapterRepository, BookRepository, NarratorRepository, SeriesRepository,
};
use podfetch_persistence::audiobookshelf::book::new_book_id;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    pub scanned_folders: usize,
    pub books_upserted: usize,
    pub books_added: usize,
    pub books_updated: usize,
    pub audio_files: usize,
    pub chapters: usize,
    pub errors: Vec<String>,
    /// Per-book results - consumers (e.g. the scan controller) emit
    /// `item_added` / `item_updated` socket.io events from these.
    pub book_results: Vec<ScannedBookResult>,
}

#[derive(Debug, Clone)]
pub struct ScannedBookResult {
    pub book_id: String,
    pub is_new: bool,
}

#[derive(Clone)]
pub struct AudiobookScanner {
    pub library_service: Arc<AudiobookshelfLibraryService>,
    pub book_repository: Arc<dyn BookRepository<Error = CustomError>>,
    pub audio_file_repository: Arc<dyn BookAudioFileRepository<Error = CustomError>>,
    pub chapter_repository: Arc<dyn BookChapterRepository<Error = CustomError>>,
    pub author_repository: Arc<dyn AuthorRepository<Error = CustomError>>,
    pub narrator_repository: Arc<dyn NarratorRepository<Error = CustomError>>,
    pub series_repository: Arc<dyn SeriesRepository<Error = CustomError>>,
    pub environment: Arc<EnvironmentService>,
}

impl AudiobookScanner {
    pub fn scan_library(&self, library_id: &str) -> Result<ScanReport, CustomError> {
        let library = self
            .library_service
            .find_by_id(library_id)?
            .ok_or_else(|| {
                common_infrastructure::error::CustomErrorInner::NotFound(
                    common_infrastructure::error::ErrorSeverity::Warning,
                )
            })?;

        let mut report = ScanReport::default();
        for folder_path in &library.folder_paths {
            let folder = PathBuf::from(folder_path);
            if !folder.is_dir() {
                report
                    .errors
                    .push(format!("not a directory: {}", folder.display()));
                continue;
            }
            for book_folder in discover_book_folders(&folder) {
                report.scanned_folders += 1;
                match self.scan_book_folder(&library.id, &book_folder) {
                    Ok(stats) => {
                        report.books_upserted += 1;
                        report.audio_files += stats.audio_files;
                        report.chapters += stats.chapters;
                        if stats.is_new {
                            report.books_added += 1;
                        } else {
                            report.books_updated += 1;
                        }
                        report.book_results.push(ScannedBookResult {
                            book_id: stats.book_id,
                            is_new: stats.is_new,
                        });
                    }
                    Err(e) => {
                        report
                            .errors
                            .push(format!("{}: {e}", book_folder.display()));
                    }
                }
            }
        }
        Ok(report)
    }

    /// Public entry point for scanning a single folder (used by upload flow + tests).
    pub fn scan_book_folder(
        &self,
        library_id: &str,
        folder: &Path,
    ) -> Result<BookScanStats, CustomError> {
        let audio_files = collect_audio_files(folder);
        if audio_files.is_empty() {
            return Err(common_infrastructure::error::CustomErrorInner::NotFound(
                common_infrastructure::error::ErrorSeverity::Warning,
            )
            .into());
        }

        // Probe every file
        let mut probed: Vec<(PathBuf, ProbedAudioFile)> = audio_files
            .into_iter()
            .filter_map(|path| match audio_probe::probe_audio_file(&path) {
                Ok(p) => Some((path, p)),
                Err(e) => {
                    tracing::warn!("ffprobe failed for {}: {e}", path.display());
                    None
                }
            })
            .collect();
        if probed.is_empty() {
            return Err(common_infrastructure::error::CustomErrorInner::NotFound(
                common_infrastructure::error::ErrorSeverity::Warning,
            )
            .into());
        }

        // Smart-track order
        sort_smart_track(&mut probed);

        // Aggregate metadata from first file (audiobookshelf behavior)
        let representative_tags = probed
            .first()
            .map(|(_, p)| p.tags.clone())
            .unwrap_or_default();
        let resolved = metadata_resolver::resolve(folder, &representative_tags);
        let total_duration: f64 = probed.iter().map(|(_, p)| p.duration).sum();

        // Upsert book row
        let now = Utc::now().naive_utc();
        let folder_path_str = folder.to_string_lossy().to_string();
        let existing = self.book_repository.find_by_folder_path(&folder_path_str)?;
        let is_new = existing.is_none();
        let book_id = existing
            .as_ref()
            .map(|b| b.id.clone())
            .unwrap_or_else(new_book_id);
        let added_at = existing.as_ref().map(|b| b.added_at).unwrap_or(now);
        let book = Book {
            id: book_id.clone(),
            library_id: library_id.to_string(),
            title: resolved.title.clone(),
            subtitle: resolved.subtitle.clone(),
            description: resolved.description.clone(),
            publisher: resolved.publisher.clone(),
            published_year: resolved.published_year.clone(),
            published_date: resolved.published_date.clone(),
            isbn: resolved.isbn.clone(),
            asin: resolved.asin.clone(),
            language: resolved.language.clone(),
            explicit: resolved.explicit,
            cover_path: existing.as_ref().and_then(|b| b.cover_path.clone()),
            duration_seconds: total_duration,
            ino: existing.as_ref().and_then(|b| b.ino.clone()),
            folder_path: folder_path_str.clone(),
            last_scan: Some(now),
            added_at,
            updated_at: now,
        };
        let book = self.book_repository.upsert(book)?;

        // Re-link authors/narrators/series
        self.author_repository.unlink_all_for_book(&book.id)?;
        for name in &resolved.authors {
            let author = self.author_repository.upsert_by_name(name)?;
            self.author_repository.link(&book.id, &author.id)?;
        }
        self.narrator_repository.unlink_all_for_book(&book.id)?;
        for name in &resolved.narrators {
            let narrator = self.narrator_repository.upsert_by_name(name)?;
            self.narrator_repository.link(&book.id, &narrator.id)?;
        }
        self.series_repository.unlink_all_for_book(&book.id)?;
        if let Some((series_name, sequence)) = &resolved.series {
            let series = self.series_repository.upsert_by_name(series_name)?;
            self.series_repository
                .link(&book.id, &series.id, sequence.as_deref())?;
        }

        // Audio files
        let mut audio_rows: Vec<BookAudioFile> = Vec::with_capacity(probed.len());
        for (idx, (path, probe)) in probed.iter().enumerate() {
            let relative = path
                .strip_prefix(folder)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            audio_rows.push(BookAudioFile {
                id: format!("af_{}", Uuid::new_v4().simple()),
                book_id: book.id.clone(),
                idx: idx as i32,
                ino: None,
                path: path.to_string_lossy().to_string(),
                relative_path: relative,
                ext: ext.clone(),
                mime_type: mime_for_ext(&ext).to_string(),
                duration: probe.duration,
                bitrate: probe.bitrate,
                codec: probe.codec.clone(),
                channels: probe.channels,
                sample_rate: probe.sample_rate,
                track_num: parse_first_int(probe.tags.track.as_deref()),
                disc_num: parse_first_int(probe.tags.disc.as_deref()),
                embedded_cover_path: None,
            });
        }
        let audio_count = audio_rows.len();
        self.audio_file_repository
            .replace_for_book(&book.id, audio_rows)?;

        // Chapters (concatenate per file with cumulative offsets; fall back to per-file titles)
        let chapters = build_chapter_list(&book.id, &probed);
        let chapter_count = chapters.len();
        self.chapter_repository
            .replace_for_book(&book.id, chapters)?;

        // Cover: prefer file in folder, fall back to ffmpeg-extracted artwork
        let cover_dir = cover_storage_dir(&self.environment);
        if let Err(e) = std::fs::create_dir_all(&cover_dir) {
            tracing::warn!("could not create cover dir {}: {e}", cover_dir.display());
        }
        let mut cover_assigned: Option<String> = None;
        if let Some(existing_cover) = cover_extractor::find_existing_cover(folder) {
            cover_assigned = Some(existing_cover.to_string_lossy().to_string());
        } else if probed.iter().any(|(_, p)| p.has_embedded_artwork) {
            let target = cover_dir.join(format!("{}.jpg", book.id));
            let source = probed
                .iter()
                .find(|(_, p)| p.has_embedded_artwork)
                .map(|(p, _)| p.clone())
                .unwrap();
            if cover_extractor::extract_embedded_cover(&source, &target).is_ok() {
                cover_assigned = Some(target.to_string_lossy().to_string());
            }
        }
        if cover_assigned.is_some() {
            let mut updated = book.clone();
            updated.cover_path = cover_assigned;
            self.book_repository.upsert(updated)?;
        }

        Ok(BookScanStats {
            audio_files: audio_count,
            chapters: chapter_count,
            is_new,
            book_id: book.id,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BookScanStats {
    pub audio_files: usize,
    pub chapters: usize,
    pub is_new: bool,
    pub book_id: String,
}

fn cover_storage_dir(env: &EnvironmentService) -> PathBuf {
    PathBuf::from(&env.audiobookshelf_data_dir).join("covers")
}

fn discover_book_folders(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Treat any sub-directory containing audio files (recursively) as a book.
            if has_audio_descendant(&path) {
                out.push(path);
            }
        }
    }
    out
}

fn has_audio_descendant(folder: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_supported_audio(&path) {
            return true;
        }
        if path.is_dir() && has_audio_descendant(&path) {
            return true;
        }
    }
    false
}

fn collect_audio_files(folder: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_supported_audio(&path) {
            out.push(path);
        } else if path.is_dir() {
            out.extend(collect_audio_files(&path));
        }
    }
    out
}

fn sort_smart_track(items: &mut [(PathBuf, ProbedAudioFile)]) {
    items.sort_by(|a, b| {
        let key_a = smart_track_key(&a.0, &a.1.tags);
        let key_b = smart_track_key(&b.0, &b.1.tags);
        key_a.cmp(&key_b)
    });
}

fn smart_track_key(path: &Path, tags: &ProbedTags) -> (i32, i32, String) {
    let disc = parse_first_int(tags.disc.as_deref()).unwrap_or(1);
    let track = parse_first_int(tags.track.as_deref())
        .or_else(|| filename_digit(path))
        .unwrap_or(0);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    (disc, track, name)
}

fn filename_digit(path: &Path) -> Option<i32> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(\d+)").unwrap());
    let name = path.file_name()?.to_str()?;
    re.captures(name)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok())
}

fn parse_first_int(value: Option<&str>) -> Option<i32> {
    let raw = value?.trim();
    // Strip "x/y" suffix
    let main = raw.split('/').next().unwrap_or(raw).trim();
    main.parse::<i32>().ok()
}

fn build_chapter_list(book_id: &str, probed: &[(PathBuf, ProbedAudioFile)]) -> Vec<BookChapter> {
    let mut chapters = Vec::new();
    let mut offset = 0.0_f64;
    let mut idx = 0_i32;

    // Case 1: at least one file contributes ffprobe chapters. Concatenate with
    // offsets; for files without chapters, synthesize one whole-file chapter.
    let any_chapters = probed.iter().any(|(_, p)| !p.chapters.is_empty());
    if any_chapters {
        for (path, p) in probed {
            if p.chapters.is_empty() {
                chapters.push(BookChapter {
                    id: format!("chp_{}", Uuid::new_v4().simple()),
                    book_id: book_id.to_string(),
                    idx,
                    start_time: offset,
                    end_time: offset + p.duration,
                    title: file_title(path, &p.tags),
                });
                idx += 1;
            } else {
                for chapter in &p.chapters {
                    chapters.push(BookChapter {
                        id: format!("chp_{}", Uuid::new_v4().simple()),
                        book_id: book_id.to_string(),
                        idx,
                        start_time: offset + chapter.start_time,
                        end_time: offset + chapter.end_time,
                        title: if chapter.title.is_empty() {
                            format!("Chapter {}", idx + 1)
                        } else {
                            chapter.title.clone()
                        },
                    });
                    idx += 1;
                }
            }
            offset += p.duration;
        }
        return chapters;
    }

    // Case 2: multi-file, no embedded chapters. If file titles are all unique
    // and non-empty, treat them as chapter titles; else use filename.
    let titles: Vec<String> = probed
        .iter()
        .map(|(path, p)| file_title(path, &p.tags))
        .collect();
    let all_unique = titles.iter().all(|t| !t.is_empty())
        && titles
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            == titles.len();
    if probed.len() > 1 && all_unique {
        for ((_, p), title) in probed.iter().zip(titles.iter()) {
            chapters.push(BookChapter {
                id: format!("chp_{}", Uuid::new_v4().simple()),
                book_id: book_id.to_string(),
                idx,
                start_time: offset,
                end_time: offset + p.duration,
                title: title.clone(),
            });
            idx += 1;
            offset += p.duration;
        }
    }
    chapters
}

fn file_title(path: &Path, tags: &ProbedTags) -> String {
    if let Some(t) = tags.title.as_deref()
        && !t.trim().is_empty()
    {
        return t.to_string();
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::audiobookshelf::audio_probe::SUPPORTED_AUDIO_EXTENSIONS;
    use std::collections::HashSet;

    #[test]
    fn smart_track_key_uses_disc_track_then_name() {
        let tags = ProbedTags {
            track: Some("3".to_string()),
            disc: Some("2".to_string()),
            ..Default::default()
        };
        let key = smart_track_key(Path::new("/x/track-03.mp3"), &tags);
        assert_eq!(key.0, 2);
        assert_eq!(key.1, 3);
    }

    #[test]
    fn parse_first_int_handles_xofy() {
        assert_eq!(parse_first_int(Some("3/10")), Some(3));
        assert_eq!(parse_first_int(Some("  42  ")), Some(42));
        assert_eq!(parse_first_int(Some("")), None);
        assert_eq!(parse_first_int(None), None);
    }

    #[test]
    fn supported_extensions_cover_common_formats() {
        let exts: HashSet<&str> = SUPPORTED_AUDIO_EXTENSIONS.iter().copied().collect();
        for required in ["mp3", "m4b", "flac", "opus", "wav", "aac"] {
            assert!(exts.contains(required), "missing required ext: {required}");
        }
    }

    #[test]
    fn build_chapter_list_synthesizes_per_file_chapters_when_unique_titles() {
        let a_tags = ProbedTags {
            title: Some("Chapter A".to_string()),
            ..Default::default()
        };
        let b_tags = ProbedTags {
            title: Some("Chapter B".to_string()),
            ..Default::default()
        };
        let probed = vec![
            (
                PathBuf::from("/x/a.mp3"),
                ProbedAudioFile {
                    duration: 100.0,
                    bitrate: 0,
                    codec: "mp3".to_string(),
                    channels: 2,
                    sample_rate: 0,
                    tags: a_tags,
                    chapters: vec![],
                    has_embedded_artwork: false,
                },
            ),
            (
                PathBuf::from("/x/b.mp3"),
                ProbedAudioFile {
                    duration: 200.0,
                    bitrate: 0,
                    codec: "mp3".to_string(),
                    channels: 2,
                    sample_rate: 0,
                    tags: b_tags,
                    chapters: vec![],
                    has_embedded_artwork: false,
                },
            ),
        ];
        let chapters = build_chapter_list("book_x", &probed);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter A");
        assert_eq!(chapters[0].start_time, 0.0);
        assert!((chapters[0].end_time - 100.0).abs() < 0.01);
        assert_eq!(chapters[1].title, "Chapter B");
        assert!((chapters[1].start_time - 100.0).abs() < 0.01);
        assert!((chapters[1].end_time - 300.0).abs() < 0.01);
    }
}
