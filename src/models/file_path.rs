use crate::DbConnection;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::service::file_service::{prepare_podcast_episode_title_to_directory};
use crate::service::path_service::PathService;

#[derive(Default, Clone, Debug)]
pub struct FilenameBuilder {
    episode: String,
    raw_episode: PodcastEpisode,
    directory: String,
    suffix: String,
    image_suffix: String,
    image_filename: String,
    filename: String,
    raw_filename: bool,
    podcast: Podcast
}

impl FilenameBuilder {

    pub fn with_podcast_directory(mut self, directory: &str) -> FilenameBuilder {
        self.directory = directory.to_string();
        self
    }

    pub fn with_episode(mut self, podcast_episode: PodcastEpisode,conn: &mut DbConnection) -> FilenameBuilder {
        self.episode = prepare_podcast_episode_title_to_directory(podcast_episode.clone(), conn);
        self.raw_episode = podcast_episode;
        self
    }

    pub fn with_filename(mut self, filename: &str) -> FilenameBuilder {
        self.filename = filename.to_string();
        self
    }

    pub fn with_image_filename(mut self, image_filename: &str) -> FilenameBuilder {
        self.image_filename = image_filename.to_string();
        self
    }

    pub fn with_suffix(mut self, suffix: &str) -> FilenameBuilder {
        self.suffix = suffix.to_string();
        self
    }

    pub fn with_image_suffix(mut self, image_suffix: &str) -> FilenameBuilder {
        self.image_suffix = image_suffix.to_string();
        self
    }

    pub fn with_raw_directory(mut self,conn: &mut DbConnection) -> FilenameBuilder {
        self.directory = PathService::get_image_path(
            &self.podcast.clone().directory_name,
            Some(self.raw_episode.clone()),
            &self.suffix,
            &self.raw_episode.name,
            conn
        );
        self.raw_filename = true;
        self
    }

    pub fn with_podcast(mut self, podcast: Podcast) -> FilenameBuilder {
        self.podcast = podcast;
        self
    }

    pub fn build(self,conn: &mut DbConnection)->(String, String){
        if self.raw_filename{
            let resulting_directory = self.clone().create_podcast_episode_dir(self.directory.clone
            (),conn);
            return (format!("{}/{}.{}", resulting_directory,self.filename.clone(), self.suffix
                .clone()),format!("{}/{}.{}", resulting_directory,self.image_filename.clone(), self.image_suffix
                .clone()));
        }
        let resulting_directory = self.clone().create_podcast_episode_dir(format!("{}/{}",self
            .directory.clone(), self.episode.clone()),conn);

        return (format!("{}/{}.{}", resulting_directory,self.filename.clone(), self.suffix.clone
        ()),format!("{}/{}.{}", resulting_directory,self.image_filename.clone(), self.image_suffix
            .clone()));
    }


    fn create_podcast_episode_dir(self,dirname:String,conn: &mut DbConnection)->String{
        PathService::check_if_podcast_episode_directory_available
            (&dirname, self.podcast, conn)
    }
}