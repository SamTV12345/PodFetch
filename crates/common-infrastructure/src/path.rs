pub fn trim_from_path(path_segment_with_extension: &str) -> (&str, &str) {
    let mut path_segment = path_segment_with_extension.split('.');
    let path_segment_first = path_segment.next().unwrap_or(path_segment_with_extension);
    (path_segment_first, path_segment.next().unwrap_or(""))
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
}
