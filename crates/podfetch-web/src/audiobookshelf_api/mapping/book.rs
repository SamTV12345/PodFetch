//! Maps a fully-hydrated `BookAggregate` to the audiobookshelf-shaped
//! LibraryItem JSON for books (`mediaType: "book"`).

use chrono::Utc;
use podfetch_domain::audiobookshelf::book::BookAggregate;
use serde_json::{Value, json};
use std::path::Path;

pub fn map_book(aggregate: &BookAggregate) -> Value {
    let book = &aggregate.book;
    let added_ms = book.added_at.and_utc().timestamp_millis();
    let updated_ms = book.updated_at.and_utc().timestamp_millis();

    let authors: Vec<Value> = aggregate
        .authors
        .iter()
        .map(|a| json!({ "id": a.id, "name": a.name }))
        .collect();
    let narrators: Vec<String> = aggregate.narrators.iter().map(|n| n.name.clone()).collect();
    let series: Vec<Value> = aggregate
        .series
        .iter()
        .map(|(s, seq)| json!({ "id": s.id, "name": s.name, "sequence": seq }))
        .collect();

    let audio_files: Vec<Value> = aggregate
        .audio_files
        .iter()
        .map(|af| {
            json!({
                "index": af.idx,
                "ino": af.ino.clone().unwrap_or_default(),
                "metadata": {
                    "path": af.path,
                    "filename": Path::new(&af.path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string(),
                    "ext": af.ext,
                    "relPath": af.relative_path,
                },
                "duration": af.duration,
                "bitRate": af.bitrate,
                "language": Value::Null,
                "codec": af.codec,
                "timeBase": "1/1000",
                "channels": af.channels,
                "channelLayout": if af.channels == 1 { "mono" } else { "stereo" },
                "trackNumFromMeta": af.track_num,
                "discNumFromMeta": af.disc_num,
                "mimeType": af.mime_type,
            })
        })
        .collect();
    let chapters: Vec<Value> = aggregate
        .chapters
        .iter()
        .map(|c| {
            json!({
                "id": c.idx,
                "start": c.start_time,
                "end": c.end_time,
                "title": c.title,
            })
        })
        .collect();

    let cover_url = format!("/api/items/{}/cover", book.id);
    let last_scan_ms = book
        .last_scan
        .map(|t| t.and_utc().timestamp_millis())
        .unwrap_or(updated_ms);

    json!({
        "id": book.id,
        "ino": book.ino.clone().unwrap_or_else(|| format!("ino_{}", book.id)),
        "libraryId": book.library_id,
        "folderId": Value::Null,
        "path": book.folder_path,
        "relPath": book.folder_path,
        "isFile": false,
        "mtimeMs": updated_ms,
        "ctimeMs": added_ms,
        "birthtimeMs": added_ms,
        "addedAt": added_ms,
        "updatedAt": updated_ms,
        "lastScan": last_scan_ms,
        "scanVersion": env!("CARGO_PKG_VERSION"),
        "isMissing": false,
        "isInvalid": false,
        "mediaType": "book",
        "media": {
            "metadata": {
                "title": book.title,
                "subtitle": book.subtitle,
                "authors": authors,
                "narrators": narrators,
                "series": series,
                "genres": [],
                "publishedYear": book.published_year,
                "publishedDate": book.published_date,
                "publisher": book.publisher,
                "description": book.description,
                "isbn": book.isbn,
                "asin": book.asin,
                "language": book.language,
                "explicit": book.explicit,
            },
            "coverPath": cover_url,
            "tags": [],
            "audioFiles": audio_files,
            "chapters": chapters,
            "duration": book.duration_seconds,
            "size": 0,
            "tracks": aggregate.audio_files.iter().enumerate().map(|(i, af)| {
                let start_offset: f64 = aggregate
                    .audio_files
                    .iter()
                    .take(i)
                    .map(|a| a.duration)
                    .sum();
                json!({
                    "index": af.idx,
                    "startOffset": start_offset,
                    "duration": af.duration,
                    "title": Path::new(&af.path).file_name().and_then(|s| s.to_str()).unwrap_or(""),
                    "contentUrl": format!("/api/items/{}/file/{}", book.id, af.idx),
                    "mimeType": af.mime_type,
                    "codec": af.codec,
                })
            }).collect::<Vec<_>>(),
        },
        "numFiles": aggregate.audio_files.len(),
        "size": 0,
        "_scanned_at_ms": Utc::now().timestamp_millis(),
    })
}
