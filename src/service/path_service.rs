pub struct PathService {

}

impl PathService {

    pub fn get_podcast_episode_path(directory: &str, episode_id: &str, suffix: &str)->String{
            return format!("podcasts\\{}\\{}\\podcast.{}", directory, episode_id, suffix);
    }
}