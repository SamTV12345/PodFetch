pub fn trim_from_path(path_segment_with_extension: &str) -> (&str, &str) {
    match path_segment_with_extension.rsplit_once('.') {
        Some((path_segment, extension))
            if path_segment_with_extension.contains('/')
                || path_segment_with_extension.contains('\\')
                || matches!(extension, "json" | "opml" | "txt" | "xml") =>
        {
            (path_segment, extension)
        }
        _ => (path_segment_with_extension, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::trim_from_path;

    #[test]
    fn test_trim_from_path() {
        assert_eq!(
            trim_from_path("src/utils/podcast_builder.rs").0,
            "src/utils/podcast_builder"
        );
    }

    #[test]
    fn test_trim_from_path_username() {
        assert_eq!(trim_from_path("max.json").0, "max");
    }

    #[test]
    fn test_trim_from_path_username_with_dot() {
        assert_eq!(
            trim_from_path("dolorem.amelie.opml"),
            ("dolorem.amelie", "opml")
        );
    }

    #[test]
    fn test_trim_from_path_username_without_extension() {
        assert_eq!(trim_from_path("dolorem.amelie"), ("dolorem.amelie", ""));
    }
}
