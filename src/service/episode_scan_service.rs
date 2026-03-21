use crate::adapters::persistence::dbconfig::db::database;
use crate::adapters::persistence::repositories::podcast_episode_chapter_repository::PodcastEpisodeChapterRepositoryImpl;
use crate::models::podcast_episode::PodcastEpisode;
use crate::service::download_service::DownloadService;
use crate::service::podcast_chapter::Chapter;
use crate::utils::error::{CustomError, CustomErrorInner, ErrorSeverity};
use file_format::FileFormat;
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};
use podfetch_web::settings::{EpisodeScanService, EpisodeWithPath, MediaFileFormat, ParsedChapter};
use std::sync::Arc;

#[derive(Clone)]
pub struct EpisodeScanServiceImpl {
    chapter_repository: Arc<dyn PodcastEpisodeChapterRepository<Error = CustomError>>,
}

impl EpisodeScanServiceImpl {
    pub fn new(
        chapter_repository: Arc<dyn PodcastEpisodeChapterRepository<Error = CustomError>>,
    ) -> Self {
        Self { chapter_repository }
    }

    pub fn default() -> Self {
        Self::new(Arc::new(PodcastEpisodeChapterRepositoryImpl::new(
            database(),
        )))
    }
}

impl EpisodeScanService for EpisodeScanServiceImpl {
    type Error = CustomError;

    fn get_episodes_with_paths_after(
        &self,
        last_id: i32,
        _limit: usize,
    ) -> Result<Vec<EpisodeWithPath>, Self::Error> {
        let episodes = PodcastEpisode::get_nth_page_of_podcast_episodes(last_id)?;
        Ok(episodes
            .into_iter()
            .filter_map(|e| {
                e.file_episode_path.map(|path| EpisodeWithPath {
                    id: e.id,
                    name: e.name,
                    file_path: path,
                })
            })
            .collect())
    }

    fn detect_file_format(&self, path: &str) -> Result<MediaFileFormat, Self::Error> {
        let detected = FileFormat::from_file(path).map_err(|e| {
            CustomErrorInner::Conflict(
                format!("Failed to detect file format: {}", e),
                ErrorSeverity::Warning,
            )
        })?;

        Ok(match detected {
            FileFormat::Mpeg12AudioLayer3
            | FileFormat::Mpeg12AudioLayer2
            | FileFormat::AppleItunesAudio
            | FileFormat::Id3v2
            | FileFormat::WaveformAudio => MediaFileFormat::Mp3,
            FileFormat::Mpeg4Part14 | FileFormat::Mpeg4Part14Audio => MediaFileFormat::Mp4,
            _ => MediaFileFormat::Unsupported,
        })
    }

    fn read_chapters_mp3(&self, path: &str) -> Result<Vec<ParsedChapter>, Self::Error> {
        let chapters = DownloadService::read_chapters_from_mp3(&path.to_string())?;
        Ok(map_chapters(chapters))
    }

    fn read_chapters_mp4(&self, path: &str) -> Vec<ParsedChapter> {
        let chapters = DownloadService::read_chapters_from_mp4(&path.to_string());
        map_chapters(chapters)
    }

    fn save_chapter(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error> {
        self.chapter_repository.upsert(chapter)
    }
}

fn map_chapters(chapters: Vec<Chapter>) -> Vec<ParsedChapter> {
    chapters
        .into_iter()
        .map(|c| ParsedChapter {
            title: c.title.unwrap_or_default(),
            start_time_seconds: c.start.num_seconds() as i32,
            end_time_seconds: c.end.map(|d| d.num_seconds() as i32).unwrap_or(0),
            href: c.link.map(|l| l.url.to_string()),
            image: c.image.map(|i| i.to_string()),
        })
        .collect()
}
