pub fn build_podcast_image_paths(
    directory: &str,
    suffix: &str,
    map_to_local_url: impl FnOnce(&str) -> String,
) -> (String, String) {
    let file_path = format!("{directory}/image.{suffix}");
    let url_path = format!("{}/image.{}", map_to_local_url(directory), suffix);
    (file_path, url_path)
}

pub fn create_available_directory<E>(
    base_path: &str,
    mut base_path_exists: impl FnMut(&str) -> bool,
    mut candidate_exists: impl FnMut(&str) -> bool,
    mut create_dir: impl FnMut(&str) -> Result<(), E>,
) -> Result<String, E> {
    if !base_path_exists(base_path) {
        create_dir(base_path)?;
        return Ok(base_path.to_string());
    }

    let mut i = 0;
    while candidate_exists(&format!("{base_path}-{i}")) {
        i += 1;
    }

    let final_path = format!("{base_path}-{i}");
    create_dir(&final_path)?;
    Ok(final_path)
}

#[cfg(test)]
mod tests {
    use super::{build_podcast_image_paths, create_available_directory};
    use std::cell::RefCell;

    #[test]
    fn builds_prefixed_podcast_image_paths() {
        let result = build_podcast_image_paths("podcasts/test", "png", |directory| {
            format!("/api/files/{directory}")
        });

        assert_eq!(
            result,
            (
                "podcasts/test/image.png".to_string(),
                "/api/files/podcasts/test/image.png".to_string()
            )
        );
    }

    #[test]
    fn creates_first_available_directory() {
        let existing = RefCell::new(vec![
            "podcasts/test".to_string(),
            "podcasts/test-0".to_string(),
        ]);

        let result = create_available_directory(
            "podcasts/test",
            |path| existing.borrow().iter().any(|item| item == path),
            |path| existing.borrow().iter().any(|item| item == path),
            |path| {
                existing.borrow_mut().push(path.to_string());
                Ok::<_, ()>(())
            },
        )
        .unwrap();

        assert_eq!(result, "podcasts/test-1");
    }
}
