//! Minimal NFO XML parser.
//!
//! audiobookshelf's `server/utils/parsers/parseNfoMetadata.js` reads a small
//! XML document with elements like `<title>`, `<author>`, `<series name>`,
//! `<description>`, `<isbn>`, `<publisher>`, `<year>`, `<language>` etc.
//! We tolerate any root element name and any case-insensitive tag.

use super::MetadataPatch;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::path::Path;

const CANDIDATE_FILENAMES: &[&str] = &["metadata.xml"];

pub fn load(folder: &Path) -> Option<MetadataPatch> {
    // First try fixed names.
    for name in CANDIDATE_FILENAMES {
        let candidate = folder.join(name);
        if candidate.is_file() {
            return std::fs::read_to_string(&candidate)
                .ok()
                .and_then(|xml| parse(&xml));
        }
    }
    // Then any *.nfo file in the folder.
    if let Ok(entries) = std::fs::read_dir(folder) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("nfo"))
                    .unwrap_or(false)
            {
                return std::fs::read_to_string(&p).ok().and_then(|xml| parse(&xml));
            }
        }
    }
    None
}

pub fn parse(xml: &str) -> Option<MetadataPatch> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut current_tag: Option<String> = None;
    let mut current_attrs: Vec<(String, String)> = Vec::new();
    let mut patch = MetadataPatch::default();
    let mut authors: Vec<String> = Vec::new();
    let mut narrators: Vec<String> = Vec::new();
    let mut series_name: Option<String> = None;
    let mut series_position: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag = std::str::from_utf8(e.name().as_ref())
                    .ok()?
                    .to_ascii_lowercase();
                current_attrs.clear();
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref())
                        .ok()?
                        .to_ascii_lowercase();
                    let value = std::str::from_utf8(&attr.value).ok()?.to_string();
                    current_attrs.push((key, value));
                }
                if tag == "series" {
                    for (k, v) in &current_attrs {
                        if k == "name" {
                            series_name = Some(v.clone());
                        } else if k == "position" || k == "number" || k == "index" {
                            series_position = Some(v.clone());
                        }
                    }
                }
                current_tag = Some(tag);
            }
            Ok(Event::Text(e)) => {
                let Some(tag) = current_tag.as_deref() else {
                    continue;
                };
                let text = e.decode().ok()?.into_owned();
                let text = unescape_xml(&text);
                let value = text.trim();
                if value.is_empty() {
                    continue;
                }
                match tag {
                    "title" => patch.title = Some(value.to_string()),
                    "subtitle" => patch.subtitle = Some(value.to_string()),
                    "description" | "plot" | "summary" => {
                        patch.description = Some(value.to_string())
                    }
                    "author" | "creator" | "writer" => authors.push(value.to_string()),
                    "narrator" | "reader" => narrators.push(value.to_string()),
                    "isbn" => patch.isbn = Some(value.to_string()),
                    "asin" => patch.asin = Some(value.to_string()),
                    "publisher" | "label" => patch.publisher = Some(value.to_string()),
                    "year" => patch.published_year = Some(value.to_string()),
                    "releasedate" | "published" | "publisheddate" => {
                        patch.published_date = Some(value.to_string());
                    }
                    "language" => patch.language = Some(value.to_string()),
                    "explicit" => {
                        patch.explicit = Some(matches!(
                            value.to_ascii_lowercase().as_str(),
                            "1" | "true" | "yes" | "explicit"
                        ));
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
                current_attrs.clear();
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
        patch.series = Some((name, series_position));
    }
    Some(patch)
}

pub(super) fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_nfo_fields() {
        let xml = r#"
            <book>
                <title>Project Hail Mary</title>
                <author>Andy Weir</author>
                <narrator>Ray Porter</narrator>
                <description>Sole survivor on a desperate mission.</description>
                <year>2021</year>
                <publisher>Ballantine Books</publisher>
                <isbn>9780593135204</isbn>
                <language>en</language>
                <series name="Solo" position="1"/>
                <explicit>false</explicit>
            </book>
        "#;
        let patch = parse(xml).unwrap();
        assert_eq!(patch.title.as_deref(), Some("Project Hail Mary"));
        assert_eq!(patch.authors, Some(vec!["Andy Weir".to_string()]));
        assert_eq!(patch.narrators, Some(vec!["Ray Porter".to_string()]));
        assert_eq!(
            patch.description.as_deref(),
            Some("Sole survivor on a desperate mission.")
        );
        assert_eq!(patch.published_year.as_deref(), Some("2021"));
        assert_eq!(patch.publisher.as_deref(), Some("Ballantine Books"));
        assert_eq!(patch.isbn.as_deref(), Some("9780593135204"));
        assert_eq!(patch.language.as_deref(), Some("en"));
        assert_eq!(
            patch.series,
            Some(("Solo".to_string(), Some("1".to_string())))
        );
        assert_eq!(patch.explicit, Some(false));
    }

    #[test]
    fn ignores_unknown_elements_gracefully() {
        let xml = r#"<doc><title>Only Title</title><random>ignored</random><authoradj>nope</authoradj></doc>"#;
        let patch = parse(xml).unwrap();
        assert_eq!(patch.title.as_deref(), Some("Only Title"));
        assert_eq!(patch.authors, None);
    }

    #[test]
    fn handles_multiple_authors_and_narrators() {
        let xml = r#"
            <doc>
                <author>A</author>
                <author>B</author>
                <narrator>X</narrator>
                <narrator>Y</narrator>
            </doc>
        "#;
        let patch = parse(xml).unwrap();
        assert_eq!(patch.authors, Some(vec!["A".to_string(), "B".to_string()]));
        assert_eq!(
            patch.narrators,
            Some(vec!["X".to_string(), "Y".to_string()])
        );
    }
}
