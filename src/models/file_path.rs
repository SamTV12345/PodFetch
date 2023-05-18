


use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::file_service::{prepare_podcast_episode_title_to_directory};
use crate::service::path_service::PathService;

#[derive(Default, Clone, Debug)]
pub struct FooBuilder {
    episode: String,
    raw_episode: PodcastEpisode,
    directory: String,
    suffix: String,
    filename: String,
    raw_filename: bool,
    podcast: Podcast
}

impl FooBuilder{

    pub fn with_podcast_directory(mut self, directory: &str) -> FooBuilder {
        self.directory = directory.to_string();
        self
    }

    pub fn with_episode(mut self, podcast_episode: PodcastEpisode) -> FooBuilder {
        self.episode = prepare_podcast_episode_title_to_directory(podcast_episode.clone());
        self.raw_episode = podcast_episode;
        self
    }

    pub fn with_filename(mut self, filename: &str) -> FooBuilder {
        self.filename = filename.to_string();
        self
    }

    pub fn with_suffix(mut self, suffix: &str) -> FooBuilder {
        self.suffix = suffix.to_string();
        self
    }

    pub fn with_raw_directory(mut self) -> FooBuilder {
        self.directory = PathService::get_image_path(
            &self.podcast.clone().directory_name,
            Some(self.raw_episode.clone()),
            &self.suffix,
            &self.raw_episode.name
        );
        self.raw_filename = true;
        self
    }

    pub fn with_podcast(mut self, podcast: Podcast) -> FooBuilder {
        self.podcast = podcast;
        self
    }

    pub fn build(self)->String{
        if self.raw_filename{
            let resulting_directory = self.clone().create_podcast_episode_dir(self.directory.clone
            ());
            return format!("{}/{}.{}", resulting_directory,self.filename.clone(), self.suffix.clone());
        }
        let resulting_directory = self.clone().create_podcast_episode_dir(format!("{}/{}",self
            .directory.clone(), self.episode.clone()));

        return format!("{}/{}.{}", resulting_directory,self.filename.clone(), self.suffix.clone());
    }


    fn create_podcast_episode_dir(self,dirname:String)->String{
        PathService::check_if_podcast_episode_directory_available
            (&dirname, self.podcast, self.filename.clone())
    }
}