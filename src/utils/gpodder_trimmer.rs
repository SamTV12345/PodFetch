pub fn trim_from_path(path_segment_with_extension: &str) -> &str {
    let path_segment = path_segment_with_extension.split('.').next().unwrap_or("");
    path_segment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_from_path() {
        let path_segment_with_extension = "src/utils/podcast_builder.rs";
        let expected = "src/utils/podcast_builder";
        let result = trim_from_path(path_segment_with_extension);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_trim_from_path_username() {
        let path_segment_with_extension = "max.json";
        let expected = "max";
        let result = trim_from_path(path_segment_with_extension);
        assert_eq!(expected, result);
    }
}