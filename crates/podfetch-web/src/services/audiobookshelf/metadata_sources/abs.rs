//! audiobookshelf-native metadata format.
//!
//! Two filenames supported:
//!   - `metadata.json` - JSON object with audiobookshelf field names
//!   - `metadata.abs` - same shape, just renamed
//!
//! Field names mirror the upstream `BookMetadata.toJSONExpanded()` so users
//! can hand-roll this file once and have it survive cross-platform syncs.

use super::MetadataPatch;
use serde_json::Value;
use std::path::Path;

const CANDIDATES: &[&str] = &["metadata.json", "metadata.abs"];

pub fn load(folder: &Path) -> Option<MetadataPatch> {
    for name in CANDIDATES {
        let p = folder.join(name);
        if p.is_file()
            && let Ok(content) = std::fs::read_to_string(&p)
            && let Some(patch) = parse(&content)
        {
            return Some(patch);
        }
    }
    None
}

pub fn parse(json: &str) -> Option<MetadataPatch> {
    let value: Value = serde_json::from_str(json).ok()?;
    let obj = value.as_object()?;
    let mut patch = MetadataPatch::default();

    if let Some(t) = string_field(obj, "title") {
        patch.title = Some(t);
    }
    if let Some(t) = string_field(obj, "subtitle") {
        patch.subtitle = Some(t);
    }
    if let Some(d) = string_field(obj, "description") {
        patch.description = Some(d);
    }
    if let Some(v) = string_field(obj, "publisher") {
        patch.publisher = Some(v);
    }
    if let Some(v) = string_field(obj, "publishedYear") {
        patch.published_year = Some(v);
    }
    if let Some(v) = string_field(obj, "publishedDate") {
        patch.published_date = Some(v);
    }
    if let Some(v) = string_field(obj, "isbn") {
        patch.isbn = Some(v);
    }
    if let Some(v) = string_field(obj, "asin") {
        patch.asin = Some(v);
    }
    if let Some(v) = string_field(obj, "language") {
        patch.language = Some(v);
    }
    if let Some(b) = obj.get("explicit").and_then(|v| v.as_bool()) {
        patch.explicit = Some(b);
    }
    if let Some(authors) = string_or_name_array(obj, "authors") {
        if !authors.is_empty() {
            patch.authors = Some(authors);
        }
    }
    if let Some(narrators) = string_or_name_array(obj, "narrators") {
        if !narrators.is_empty() {
            patch.narrators = Some(narrators);
        }
    }
    if let Some(series_value) = obj.get("series") {
        patch.series = parse_series(series_value);
    }

    Some(patch)
}

fn string_field(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Option<String> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn string_or_name_array(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Option<Vec<String>> {
    let v = obj.get(key)?;
    if let Some(arr) = v.as_array() {
        let mut out = Vec::new();
        for entry in arr {
            if let Some(s) = entry.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            } else if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            }
        }
        return Some(out);
    }
    if let Some(s) = v.as_str() {
        return Some(
            s.split([',', ';'])
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect(),
        );
    }
    None
}

fn parse_series(value: &Value) -> Option<(String, Option<String>)> {
    if let Some(s) = value.as_str() {
        return Some((s.to_string(), None));
    }
    if let Some(obj) = value.as_object() {
        let name = obj.get("name").and_then(|n| n.as_str())?;
        let seq = obj
            .get("sequence")
            .or_else(|| obj.get("position"))
            .and_then(|n| n.as_str())
            .map(str::to_string)
            .or_else(|| {
                obj.get("sequence")
                    .or_else(|| obj.get("position"))
                    .and_then(|n| n.as_f64())
                    .map(|f| {
                        if f.fract() == 0.0 {
                            format!("{}", f as i64)
                        } else {
                            format!("{f}")
                        }
                    })
            });
        return Some((name.to_string(), seq));
    }
    if let Some(arr) = value.as_array() {
        if let Some(first) = arr.first() {
            return parse_series(first);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_metadata_json() {
        let json = r#"{
            "title": "The Witcher",
            "subtitle": "The Last Wish",
            "authors": [{"name": "Andrzej Sapkowski"}],
            "narrators": ["Peter Kenny"],
            "series": [{"name": "Witcher", "sequence": "0"}],
            "publishedYear": "2007",
            "publishedDate": "2007-12-01",
            "publisher": "Gollancz",
            "isbn": "9780316029186",
            "asin": "B005LAIHV4",
            "language": "en",
            "explicit": false,
            "description": "Geralt of Rivia."
        }"#;
        let patch = parse(json).unwrap();
        assert_eq!(patch.title.as_deref(), Some("The Witcher"));
        assert_eq!(patch.subtitle.as_deref(), Some("The Last Wish"));
        assert_eq!(
            patch.authors,
            Some(vec!["Andrzej Sapkowski".to_string()])
        );
        assert_eq!(patch.narrators, Some(vec!["Peter Kenny".to_string()]));
        assert_eq!(
            patch.series,
            Some(("Witcher".to_string(), Some("0".to_string())))
        );
        assert_eq!(patch.published_year.as_deref(), Some("2007"));
        assert_eq!(patch.publisher.as_deref(), Some("Gollancz"));
        assert_eq!(patch.isbn.as_deref(), Some("9780316029186"));
        assert_eq!(patch.asin.as_deref(), Some("B005LAIHV4"));
        assert_eq!(patch.language.as_deref(), Some("en"));
        assert_eq!(patch.explicit, Some(false));
    }

    #[test]
    fn accepts_string_authors_and_comma_separated_narrators() {
        let json = r#"{
            "title": "X",
            "authors": "Author A, Author B",
            "narrators": "Reader 1; Reader 2"
        }"#;
        let patch = parse(json).unwrap();
        assert_eq!(
            patch.authors,
            Some(vec!["Author A".to_string(), "Author B".to_string()])
        );
        assert_eq!(
            patch.narrators,
            Some(vec!["Reader 1".to_string(), "Reader 2".to_string()])
        );
    }

    #[test]
    fn series_can_be_object_or_plain_string() {
        let plain = parse(r#"{"title": "T", "series": "Just A Series"}"#).unwrap();
        assert_eq!(plain.series, Some(("Just A Series".to_string(), None)));
        let object = parse(
            r#"{"title": "T", "series": {"name": "X", "sequence": 3.5}}"#,
        )
        .unwrap();
        assert_eq!(
            object.series,
            Some(("X".to_string(), Some("3.5".to_string())))
        );
    }

    #[test]
    fn malformed_json_returns_none() {
        assert!(parse("not json").is_none());
        assert!(parse("[]").is_none()); // not an object
    }
}
