pub fn trim_from_path(path_segment_with_extension: &str) -> (&str, &str) {
    let mut path_segment = path_segment_with_extension.split(".");

    let path_segment_first = path_segment.next().unwrap_or(path_segment_with_extension);
    (path_segment_first, path_segment.next().unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_trim_from_path() {
        let path_segment_with_extension = "src/utils/podcast_builder.rs";
        let expected = "src/utils/podcast_builder";
        let result = trim_from_path(path_segment_with_extension);
        assert_eq!(expected, result.0);
    }

    #[test]
    #[serial]
    fn test_trim_from_path_username() {
        let path_segment_with_extension = "max.json";
        let expected = "max";
        let result = trim_from_path(path_segment_with_extension);
        assert_eq!(expected, result.0);
    }
}
