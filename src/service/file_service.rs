use std::path::Path;


pub fn check_if_podcast_episode_downloaded(podcast_id: i64, episode_id: String) -> bool {
    return Path::new(&format!("episodes\\{}", episode_id)).exists()
}

pub fn create_podcast_root_directory_exists(){
    if !Path::new("podcasts").exists() {
        std::fs::create_dir("podcasts").expect("Error creating directory");
    }
}