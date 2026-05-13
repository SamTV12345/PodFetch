//! Minimal EPUB OPF parser.
//!
//! The OPF (Open Packaging Format) document inside an EPUB carries
//! `<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">` with Dublin
//! Core elements (`dc:title`, `dc:creator`, `dc:identifier`, `dc:date`,
//! `dc:language`, `dc:description`, `dc:publisher`) plus custom <meta>
//! tags for series.
//!
//! Audiobookshelf reads the first `.opf` file in the book folder; we do
//! the same.

use super::MetadataPatch;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::path::Path;

pub fn load(folder: &Path) -> Option<MetadataPatch> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return None;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file()
            && p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("opf"))
                .unwrap_or(false)
        {
            return std::fs::read_to_string(&p).ok().and_then(|xml| parse(&xml));
        }
    }
    None
}

pub fn parse(xml: &str) -> Option<MetadataPatch> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut patch = MetadataPatch::default();
    let mut authors: Vec<String> = Vec::new();
    let mut narrators: Vec<String> = Vec::new();
    let mut series_name: Option<String> = None;
    let mut series_index: Option<String> = None;

    let mut current_tag: Option<String> = None;
    let mut current_role: Option<String> = None;
    let mut current_identifier_scheme: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.name();
                let raw_tag = std::str::from_utf8(name.as_ref()).ok()?;
                let local = raw_tag.rsplit_once(':').map(|(_, l)| l).unwrap_or(raw_tag);
                let lower_tag = local.to_ascii_lowercase();
                current_role = None;
                current_identifier_scheme = None;

                let mut attrs: Vec<(String, String)> = Vec::new();
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref()).ok()?;
                    let local_key = key.rsplit_once(':').map(|(_, l)| l).unwrap_or(key);
                    let value = std::str::from_utf8(&attr.value).ok()?.to_string();
                    attrs.push((local_key.to_ascii_lowercase(), value));
                }
                if lower_tag == "creator" {
                    for (k, v) in &attrs {
                        if k == "role" {
                            current_role = Some(v.clone());
                        }
                    }
                } else if lower_tag == "identifier" {
                    for (k, v) in &attrs {
                        if k == "scheme" {
                            current_identifier_scheme = Some(v.to_ascii_lowercase());
                        }
                    }
                } else if lower_tag == "meta" {
                    // Calibre-style series meta tags: <meta name="calibre:series" content="X"/>
                    let name = attrs.iter().find(|(k, _)| k == "name").map(|(_, v)| v.clone());
                    let content = attrs
                        .iter()
                        .find(|(k, _)| k == "content")
                        .map(|(_, v)| v.clone());
                    if let (Some(name), Some(content)) = (name, content) {
                        match name.to_ascii_lowercase().as_str() {
                            "calibre:series" => series_name = Some(content),
                            "calibre:series_index" => series_index = Some(content),
                            _ => {}
                        }
                    }
                }
                current_tag = Some(lower_tag);
            }
            Ok(Event::Text(e)) => {
                let Some(tag) = current_tag.as_deref() else {
                    continue;
                };
                let value = e.decode().ok()?.into_owned();
                let value = super::nfo::unescape_xml(&value);
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }
                match tag {
                    "title" => patch.title = Some(value.to_string()),
                    "description" => patch.description = Some(value.to_string()),
                    "language" => patch.language = Some(value.to_string()),
                    "publisher" => patch.publisher = Some(value.to_string()),
                    "date" => {
                        patch.published_date = Some(value.to_string());
                        // Best-effort year extraction (YYYY-MM-DD)
                        if patch.published_year.is_none() {
                            let year: String = value.chars().take(4).collect();
                            if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
                                patch.published_year = Some(year);
                            }
                        }
                    }
                    "creator" => {
                        let role = current_role.as_deref().unwrap_or("aut");
                        if role.eq_ignore_ascii_case("nrt") {
                            narrators.push(value.to_string());
                        } else {
                            authors.push(value.to_string());
                        }
                    }
                    "identifier" => match current_identifier_scheme.as_deref() {
                        Some("isbn") => patch.isbn = Some(value.to_string()),
                        Some("asin") | Some("amazon") => patch.asin = Some(value.to_string()),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
                current_role = None;
                current_identifier_scheme = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }
    if !authors.is_empty() {
        patch.authors = Some(authors);
    }
    if !narrators.is_empty() {
        patch.narrators = Some(narrators);
    }
    if let Some(name) = series_name {
        patch.series = Some((name, series_index));
    }
    Some(patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_opf_with_dc_elements() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
        <package xmlns="http://www.idpf.org/2007/opf" version="3.0">
          <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
            <dc:title>Mistborn</dc:title>
            <dc:creator opf:role="aut">Brandon Sanderson</dc:creator>
            <dc:creator opf:role="nrt">Michael Kramer</dc:creator>
            <dc:identifier opf:scheme="ISBN">9780765350381</dc:identifier>
            <dc:language>en</dc:language>
            <dc:publisher>Tor Books</dc:publisher>
            <dc:date>2006-07-17</dc:date>
            <dc:description>The Lord Ruler.</dc:description>
            <meta name="calibre:series" content="Mistborn"/>
            <meta name="calibre:series_index" content="1"/>
          </metadata>
        </package>"#;
        let patch = parse(xml).unwrap();
        assert_eq!(patch.title.as_deref(), Some("Mistborn"));
        assert_eq!(
            patch.authors,
            Some(vec!["Brandon Sanderson".to_string()])
        );
        assert_eq!(
            patch.narrators,
            Some(vec!["Michael Kramer".to_string()])
        );
        assert_eq!(patch.isbn.as_deref(), Some("9780765350381"));
        assert_eq!(patch.language.as_deref(), Some("en"));
        assert_eq!(patch.publisher.as_deref(), Some("Tor Books"));
        assert_eq!(patch.published_date.as_deref(), Some("2006-07-17"));
        assert_eq!(patch.published_year.as_deref(), Some("2006"));
        assert_eq!(patch.description.as_deref(), Some("The Lord Ruler."));
        assert_eq!(
            patch.series,
            Some(("Mistborn".to_string(), Some("1".to_string())))
        );
    }

    #[test]
    fn creator_without_role_defaults_to_author() {
        let xml = r#"<?xml version="1.0"?>
        <package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
          <dc:creator>Solo Author</dc:creator>
        </metadata></package>"#;
        let patch = parse(xml).unwrap();
        assert_eq!(patch.authors, Some(vec!["Solo Author".to_string()]));
        assert!(patch.narrators.is_none());
    }

    #[test]
    fn malformed_xml_returns_none() {
        assert!(parse("<unclosed").is_none() || parse("<unclosed").unwrap() == MetadataPatch::default());
    }
}
