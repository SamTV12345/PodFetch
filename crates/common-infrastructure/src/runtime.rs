use std::sync::LazyLock;

use crate::config::EnvironmentService;
use crate::logging::init_logging;

pub const ITUNES_URL: &str = "https://itunes.apple.com/search";
pub const PODCAST_FILENAME: &str = "podcast";
pub const PODCAST_IMAGENAME: &str = "image";
pub const DEFAULT_DEVICE: &str = "webview";
pub const DEFAULT_IMAGE_URL: &str = "ui/default.jpg";
pub const MAIN_ROOM: &str = "main";

/// Whether a stored image URL stands for "no image of its own": empty, or
/// PodFetch's default image.
pub fn is_default_image_url(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.is_empty()
        || trimmed == DEFAULT_IMAGE_URL
        || trimmed == format!("/{DEFAULT_IMAGE_URL}")
        || trimmed.ends_with(&format!("/{DEFAULT_IMAGE_URL}"))
}

pub static ENVIRONMENT_SERVICE: LazyLock<EnvironmentService> = LazyLock::new(|| {
    init_logging();
    #[cfg(test)]
    {
        let env = EnvironmentService::for_tests();
        println!("Environment: {:?}", env.database_url);
        env
    }
    #[cfg(not(test))]
    EnvironmentService::new()
});

#[cfg(test)]
mod default_image_tests {
    use super::is_default_image_url;

    #[test]
    fn empty_and_default_urls_are_the_default_image() {
        assert!(is_default_image_url(""));
        assert!(is_default_image_url("   "));
        assert!(is_default_image_url("ui/default.jpg"));
        assert!(is_default_image_url("/ui/default.jpg"));
        assert!(is_default_image_url("http://localhost:8000/ui/default.jpg"));
        assert!(!is_default_image_url("https://example.com/cover.jpg"));
    }
}
