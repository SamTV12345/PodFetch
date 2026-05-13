//! `desc.txt` / `reader.txt` source.
//!
//! audiobookshelf reads:
//!   - `desc.txt` -> description (full file content trimmed)
//!   - `reader.txt` -> narrators (first non-empty line, comma-/semicolon-split)

use super::MetadataPatch;
use std::path::Path;

pub fn load(folder: &Path) -> Option<MetadataPatch> {
    let mut patch = MetadataPatch::default();
    let mut touched = false;

    let desc = folder.join("desc.txt");
    if desc.is_file()
        && let Ok(content) = std::fs::read_to_string(&desc)
    {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            patch.description = Some(trimmed.to_string());
            touched = true;
        }
    }
    let reader = folder.join("reader.txt");
    if reader.is_file()
        && let Ok(content) = std::fs::read_to_string(&reader)
    {
        if let Some(narrators) = first_line_split(&content) {
            patch.narrators = Some(narrators);
            touched = true;
        }
    }
    touched.then_some(patch)
}

fn first_line_split(content: &str) -> Option<Vec<String>> {
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let split: Vec<String> = trimmed
                .split([',', ';'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !split.is_empty() {
                return Some(split);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_first_line_by_commas() {
        let narrators = first_line_split("Andy Weir, Ray Porter\nlater line\n").unwrap();
        assert_eq!(
            narrators,
            vec!["Andy Weir".to_string(), "Ray Porter".to_string()]
        );
    }

    #[test]
    fn skips_blank_lines() {
        let narrators = first_line_split("\n\n   \nFirst, Second\n").unwrap();
        assert_eq!(narrators, vec!["First".to_string(), "Second".to_string()]);
    }

    #[test]
    fn semicolon_separator() {
        let narrators = first_line_split("A; B; C").unwrap();
        assert_eq!(narrators.len(), 3);
        assert_eq!(narrators[2], "C");
    }
}
