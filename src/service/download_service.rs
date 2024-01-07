use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::file_service::FileService;

use reqwest::blocking::ClientBuilder;

use id3::{ErrorKind, Tag, TagLike, Version};
use reqwest::header::HeaderMap;
use std::io;
use std::io::Read;

use crate::config::dbconfig::establish_connection;
use crate::constants::inner_constants::{DEFAULT_IMAGE_URL, PODCAST_FILENAME, PODCAST_IMAGENAME};
use crate::dbconfig::DBType;
use crate::get_default_image;
use crate::models::file_path::{FilenameBuilder, FilenameBuilderReturn};
use crate::models::settings::Setting;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;
use crate::utils::error::CustomError;
use crate::utils::file_extension_determination::{determine_file_extension, FileType};

pub struct DownloadService {
    pub client_builder: ClientBuilder,
    pub file_service: FileService,
}

impl DownloadService {
    pub fn new() -> Self {
        DownloadService {
            client_builder: ClientBuilder::new(),
            file_service: FileService::new(),
        }
    }

    pub fn download_podcast_episode(
        &mut self,
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
    ) -> Result<(), CustomError> {
        let client = ClientBuilder::new().build().unwrap();

        let conn = &mut establish_connection();
        let suffix = determine_file_extension(&podcast_episode.url, &client, FileType::Audio);
        let settings_in_db = Setting::get_settings(conn)?.unwrap();
        let image_suffix =
            determine_file_extension(&podcast_episode.image_url, &client, FileType::Image);

        let mut header_map = HeaderMap::new();
        add_basic_auth_headers_conditionally(podcast_episode.url.clone(), &mut header_map);
        let mut resp = client
            .get(podcast_episode.url.clone())
            .headers(header_map.clone())
            .send()
            .unwrap();

        let mut image_response;
        match podcast_episode.image_url == DEFAULT_IMAGE_URL {
            true=>{
                image_response = client
                    .get(get_default_image())
                    .headers(header_map)
                    .send()
                    .unwrap();
            }
            false=>{
                image_response = client
                    .get(podcast_episode.image_url.clone())
                    .headers(header_map)
                    .send()
                    .unwrap();
            }
        }

        let paths = match settings_in_db.use_existing_filename {
            true => FilenameBuilder::default()
                .with_podcast(podcast.clone())
                .with_suffix(&suffix)
                .with_settings(settings_in_db)
                .with_episode(podcast_episode.clone(), conn)?
                .with_filename(PODCAST_FILENAME)
                .with_image_filename(PODCAST_IMAGENAME)
                .with_image_suffix(&image_suffix)
                .with_raw_directory(conn)?
                .build(conn)?,
            false => FilenameBuilder::default()
                .with_suffix(&suffix)
                .with_settings(settings_in_db)
                .with_image_suffix(&image_suffix)
                .with_episode(podcast_episode.clone(), conn)?
                .with_podcast_directory(&podcast.directory_name)
                .with_podcast(podcast.clone())
                .with_image_filename(PODCAST_IMAGENAME)
                .with_filename(PODCAST_FILENAME)
                .build(conn)?,
        };

        let mut podcast_out = std::fs::File::create(&paths.filename).unwrap();
        let mut image_out = std::fs::File::create(&paths.image_filename).unwrap();

        if !self
            .file_service
            .check_if_podcast_main_image_downloaded(&podcast.clone().directory_id, conn)
        {
            let mut image_podcast = std::fs::File::create(&paths.image_filename).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");

        PodcastEpisode::update_local_paths(
            &podcast_episode.episode_id,
            &paths.local_image_url,
            &paths.local_file_url,
            &paths.image_filename,
            &paths.filename,
            conn,
        )?;
        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
        Self::update_meta_data(paths, podcast_episode, podcast, conn)?;
        Ok(())
    }

    fn update_meta_data(
        paths: FilenameBuilderReturn,
        podcast_episode: PodcastEpisode,
        podcast: Podcast,
        conn: &mut DBType,
    ) -> Result<(), CustomError> {
        let mut tag = match Tag::read_from_path(&paths.filename) {
            Ok(tag) => tag,
            Err(id3::Error {
                kind: ErrorKind::NoTag,
                ..
            }) => Tag::new(),
            Err(err) => return Err(CustomError::Conflict(err.to_string())),
        };

        if let 0 = tag.pictures().count() {
            let mut image_file = std::fs::File::open(paths.image_filename).unwrap();
            let mut image_data = Vec::new();
            let _ = image_file.read_to_end(&mut image_data);
            tag.add_frame(id3::frame::Picture {
                mime_type: "image/jpeg".to_string(),
                picture_type: id3::frame::PictureType::CoverFront,
                description: "Cover".to_string(),
                data: image_data,
            });
        }

        if tag.title().is_none() {
            tag.set_title(podcast_episode.name);
        }

        if tag.artist().is_none() {
            if let Some(author) = podcast.author {
                tag.set_artist(author);
            }
        }

        if tag.album().is_none() {
            tag.set_album(podcast.name);
        }

        tag.set_date_recorded(podcast_episode.date_of_recording.parse().unwrap());

        if tag.genres().is_none() {
            if let Some(keywords) = podcast.keywords {
                tag.set_genre(keywords);
            }
        }

        if tag.clone().comments().next().is_none() {
            tag.add_frame(id3::frame::Comment {
                lang: podcast.language.unwrap_or("eng".to_string()),
                description: "Comment".to_string(),
                text: podcast_episode.description,
            });
        }

        let track_number = PodcastEpisodeService::get_track_number_for_episode(
            conn,
            podcast.id,
            &podcast_episode.date_of_recording,
        );

        if tag.track().is_none() {
            if let Ok(track_number) = track_number {
                tag.set_track(track_number as u32);
            }
        }

        tag.write_to_path(paths.filename, Version::Id3v24)
            .map(|_| ())
            .map_err(|e| CustomError::Conflict(e.to_string()))?;

        Ok(())
    }
}
