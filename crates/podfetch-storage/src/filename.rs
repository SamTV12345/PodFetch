#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct FilenameBuilder {
    episode_stem: String,
    podcast_directory: String,
    suffix: String,
    image_suffix: String,
    image_filename: String,
    filename: String,
    direct_paths: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilenameBuilderReturn {
    pub filename: String,
    pub image_filename: String,
}

impl FilenameBuilderReturn {
    pub fn new(filename: String, image_filename: String) -> Self {
        Self {
            filename,
            image_filename,
        }
    }
}

impl FilenameBuilder {
    pub fn with_podcast_directory(mut self, directory: &str) -> Self {
        self.podcast_directory = directory.to_string();
        self
    }

    pub fn with_episode_stem(mut self, episode_stem: &str) -> Self {
        self.episode_stem = episode_stem.to_string();
        self
    }

    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }

    pub fn with_image_filename(mut self, image_filename: &str) -> Self {
        self.image_filename = image_filename.to_string();
        self
    }

    pub fn with_suffix(mut self, suffix: &str) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    pub fn with_image_suffix(mut self, image_suffix: &str) -> Self {
        self.image_suffix = image_suffix.to_string();
        self
    }

    pub fn with_direct_paths(mut self, direct_paths: bool) -> Self {
        self.direct_paths = direct_paths;
        self
    }

    pub fn build<E>(
        self,
        resolve_directory: impl FnOnce(String) -> Result<String, E>,
    ) -> Result<FilenameBuilderReturn, E> {
        if self.direct_paths {
            return Ok(FilenameBuilderReturn::new(
                format!(
                    "{}/{}.{}",
                    self.podcast_directory, self.episode_stem, self.suffix
                ),
                format!(
                    "{}/{}.{}",
                    self.podcast_directory, self.episode_stem, self.image_suffix
                ),
            ));
        }

        let resulting_directory =
            resolve_directory(format!("{}/{}", self.podcast_directory, self.episode_stem))?;

        Ok(FilenameBuilderReturn::new(
            format!("{}/{}.{}", resulting_directory, self.filename, self.suffix),
            format!(
                "{}/{}.{}",
                resulting_directory, self.image_filename, self.image_suffix
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{FilenameBuilder, FilenameBuilderReturn};

    #[test]
    fn builds_direct_paths() {
        let paths = FilenameBuilder::default()
            .with_podcast_directory("podcasts/test")
            .with_episode_stem("episode-1")
            .with_suffix("mp3")
            .with_image_suffix("jpg")
            .with_direct_paths(true)
            .build::<()>(Ok)
            .unwrap();

        assert_eq!(
            paths,
            FilenameBuilderReturn::new(
                "podcasts/test/episode-1.mp3".to_string(),
                "podcasts/test/episode-1.jpg".to_string()
            )
        );
    }

    #[test]
    fn builds_nested_paths() {
        let paths = FilenameBuilder::default()
            .with_podcast_directory("podcasts/test")
            .with_episode_stem("episode-1")
            .with_filename("audio")
            .with_image_filename("image")
            .with_suffix("mp3")
            .with_image_suffix("jpg")
            .build(|directory| Ok::<_, ()>(format!("{directory}-1")))
            .unwrap();

        assert_eq!(
            paths,
            FilenameBuilderReturn::new(
                "podcasts/test/episode-1-1/audio.mp3".to_string(),
                "podcasts/test/episode-1-1/image.jpg".to_string()
            )
        );
    }
}
