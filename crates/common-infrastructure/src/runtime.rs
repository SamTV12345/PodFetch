use std::sync::LazyLock;

use crate::config::EnvironmentService;
use crate::logging::init_logging;

pub const ITUNES_URL: &str = "https://itunes.apple.com/search";
pub const PODCAST_FILENAME: &str = "podcast";
pub const PODCAST_IMAGENAME: &str = "image";
pub const DEFAULT_DEVICE: &str = "webview";
pub const DEFAULT_IMAGE_URL: &str = "ui/default.jpg";
pub const MAIN_ROOM: &str = "main";

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
