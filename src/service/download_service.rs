use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::file_service::FileService;
use std::fs::File;

use reqwest::blocking::ClientBuilder;

use crate::adapters::file::file_handler::FileHandlerType;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::constants::inner_constants::{ENVIRONMENT_SERVICE, PODCAST_FILENAME, PODCAST_IMAGENAME};
use crate::models::file_path::{FilenameBuilder, FilenameBuilderReturn};
use crate::models::podcast_settings::PodcastSetting;
use crate::models::settings::Setting;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{map_reqwest_error, CustomError, CustomErrorInner};
use crate::utils::file_extension_determination::{
    determine_file_extension, DetermineFileExtensionReturn, FileType,
};
use crate::utils::http_client::get_async_sync_client;
use crate::utils::reqwest_client::get_sync_client;
use file_format::FileFormat;
use id3::{ErrorKind, Tag, TagLike, Version};
use std::io::Read;

pub struct DownloadService {}

impl DownloadService {
    pub fn handle_suffix_response(
        dt: DetermineFileExtensionReturn,
        podcast_episode_url: &str,
    ) -> Result<(String, Vec<u8>), CustomError> {
        match dt {
            DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
            DetermineFileExtensionReturn::String(suffix) => {
                let resp = get_sync_client()
                    .build()
                    .map_err(map_reqwest_error)?
                    .get(podcast_episode_url)
                    .send()
                    .map_err(map_reqwest_error)?
                    .bytes()
                    .map_err(map_reqwest_error)?
                    .as_ref()
                    .to_vec();
                Ok((suffix, resp))
            }
        }
    }

    pub async fn handle_suffix_response_async(
        dt: DetermineFileExtensionReturn,
        podcast_episode_url: &str,
    ) -> Result<(String, Vec<u8>), CustomError> {
        match dt {
            DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
            DetermineFileExtensionReturn::String(suffix) => {
                let resp = get_async_sync_client()
                    .build()
                    .map_err(map_reqwest_error)?
                    .get(podcast_episode_url)
                    .send()
                    .await
                    .map_err(map_reqwest_error)?
                    .bytes()
                    .await
                    .map_err(map_reqwest_error)?
                    .as_ref()
                    .to_vec();
                Ok((suffix, resp))
            }
        }
    }

    pub fn download_podcast_episode(
        podcast_episode: PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<(), CustomError> {
        let client = ClientBuilder::new().build().unwrap();
        let conn = &mut get_connection();
        let mut podcast_data = Self::handle_suffix_response(
            determine_file_extension(&podcast_episode.url, &client, FileType::Audio),
            &podcast_episode.url,
        )?;
        let settings_in_db = Setting::get_settings()?.unwrap();
        let mut image_data = Self::handle_suffix_response(
            determine_file_extension(&podcast_episode.image_url, &client, FileType::Image),
            &podcast_episode.image_url,
        )?;

        let paths = match settings_in_db.use_existing_filename {
            true => FilenameBuilder::default()
                .with_podcast(podcast.clone())
                .with_suffix(&podcast_data.0)
                .with_settings(settings_in_db)
                .with_episode(podcast_episode.clone())?
                .with_filename(PODCAST_FILENAME)
                .with_image_filename(PODCAST_IMAGENAME)
                .with_image_suffix(&image_data.0)
                .with_raw_directory()?
                .build(conn)?,
            false => FilenameBuilder::default()
                .with_suffix(&podcast_data.0)
                .with_settings(settings_in_db)
                .with_image_suffix(&image_data.0)
                .with_episode(podcast_episode.clone())?
                .with_podcast_directory(&podcast.directory_name)
                .with_podcast(podcast.clone())
                .with_image_filename(PODCAST_IMAGENAME)
                .with_filename(PODCAST_FILENAME)
                .build(conn)?,
        };

        if !FileService::check_if_podcast_main_image_downloaded(&podcast.clone().directory_id, conn)
        {
            ENVIRONMENT_SERVICE
                .default_file_handler
                .write_file(&paths.image_filename, image_data.1.as_mut_slice())?;
        }

        ENVIRONMENT_SERVICE
            .default_file_handler
            .write_file(&paths.filename, podcast_data.1.as_mut_slice())?;

        PodcastEpisode::update_local_paths(
            &podcast_episode.episode_id,
            &paths.image_filename,
            &paths.filename,
            conn,
        )?;
        ENVIRONMENT_SERVICE
            .default_file_handler
            .write_file(&paths.image_filename, image_data.1.as_mut_slice())?;
        let result = Self::handle_metadata_insertion(&paths, &podcast_episode, podcast);
        if let Err(err) = result {
            log::error!("Error handling metadata insertion: {:?}", err);
        }
        Ok(())
    }

    pub fn handle_metadata_insertion(
        paths: &FilenameBuilderReturn,
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<(), CustomError> {
        if ENVIRONMENT_SERVICE.default_file_handler.get_type() == FileHandlerType::S3 {
            return Ok(());
        }

        let detected_file = FileFormat::from_file(&paths.filename).unwrap();

        match detected_file {
            FileFormat::Mpeg12AudioLayer3
            | FileFormat::Mpeg12AudioLayer2
            | FileFormat::AppleItunesAudio
            | FileFormat::Id3v2
            | FileFormat::WaveformAudio => {
                let result_of_update = Self::update_meta_data_mp3(paths, podcast_episode, podcast);
                if let Some(err) = result_of_update.err() {
                    log::error!("Error updating metadata: {:?}", err);
                }
            }
            FileFormat::Mpeg4Part14 | FileFormat::Mpeg4Part14Audio => {
                let result_of_update = Self::update_meta_data_mp4(paths, podcast_episode, podcast);
                if let Some(err) = result_of_update.err() {
                    log::error!("Error updating metadata: {:?}", err);
                }
            }
            _ => {
                log::error!("File format not supported: {:?}", detected_file);
                return Err(
                    CustomErrorInner::Conflict("File format not supported".to_string()).into(),
                );
            }
        }
        Ok(())
    }

    fn update_meta_data_mp3(
        paths: &FilenameBuilderReturn,
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<(), CustomError> {
        let mut tag = match Tag::read_from_path(&paths.filename) {
            Ok(tag) => tag,
            Err(id3::Error {
                kind: ErrorKind::NoTag,
                ..
            }) => Tag::new(),
            Err(err) => return Err(CustomErrorInner::Conflict(err.to_string()).into()),
        };

        if let Version::Id3v22 = tag.version() {
            tag = Tag::new();
        }

        if let 0 = tag.pictures().count() {
            let mut image_file = File::open(&paths.image_filename).unwrap();
            let mut image_data = Vec::new();
            let _ = image_file.read_to_end(&mut image_data);
            tag.add_frame(id3::frame::Picture {
                mime_type: "image/jpeg".to_string(),
                picture_type: id3::frame::PictureType::CoverFront,
                description: "Cover".to_string(),
                data: image_data,
            });
        }

        let mut conn = get_connection();
        let index = PodcastEpisode::get_position_of_episode(
            &podcast_episode.date_of_recording,
            podcast_episode.podcast_id,
            &mut conn,
        )?;

        let settings_for_podcast = PodcastSetting::get_settings(podcast.id)?;

        if let Some(settings_for_podcast) = settings_for_podcast {
            if settings_for_podcast.episode_numbering {
                if !podcast_episode.episode_numbering_processed {
                    tag.set_title(format!("{} - {}", index, &podcast_episode.name));
                    PodcastEpisode::update_episode_numbering_processed(
                        &mut conn,
                        true,
                        &podcast_episode.episode_id,
                    );
                }
            } else {
                tag.set_title(&podcast_episode.name);
                PodcastEpisode::update_episode_numbering_processed(
                    &mut conn,
                    false,
                    &podcast_episode.episode_id,
                )
            }
        } else {
            tag.set_title(&podcast_episode.name);
            PodcastEpisode::update_episode_numbering_processed(
                &mut conn,
                false,
                &podcast_episode.episode_id,
            )
        }

        if tag.artist().is_none() {
            if let Some(author) = &podcast.author {
                tag.set_artist(author);
            }
        }

        if tag.album().is_none() {
            tag.set_album(&podcast.name);
        }

        tag.set_date_recorded(podcast_episode.date_of_recording.parse().unwrap());

        if tag.genres().is_none() {
            if let Some(keywords) = &podcast.keywords {
                tag.set_genre(keywords);
            }
        }

        if tag.clone().comments().next().is_none() {
            tag.add_frame(id3::frame::Comment {
                lang: podcast.clone().language.unwrap_or("eng".to_string()),
                description: "Comment".to_string(),
                text: podcast_episode.clone().description,
            });
        }

        let track_number = PodcastEpisodeService::get_track_number_for_episode(
            podcast.id,
            &podcast_episode.date_of_recording,
        );

        if tag.track().is_none() {
            if let Ok(track_number) = track_number {
                tag.set_track(track_number as u32);
            }
        }

        let write_succesful: Result<(), CustomError> = tag
            .write_to_path(&paths.filename, Version::Id3v24)
            .map(|_| ())
            .map_err(|e| CustomErrorInner::Conflict(e.to_string()).into());

        if write_succesful.is_err() {
            log::error!(
                "Error writing metadata: {:?}",
                write_succesful.err().unwrap()
            );
        }
        Ok(())
    }

    fn update_meta_data_mp4(
        paths: &FilenameBuilderReturn,
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<(), CustomError> {
        let tag = mp4ameta::Tag::read_from_path(&paths.filename);
        match tag {
            Ok(mut tag) => {
                tag.set_title(&podcast_episode.name);
                tag.set_artist(podcast.clone().author.unwrap_or("Unknown".to_string()));
                tag.set_album(&podcast.name);
                tag.set_genre(podcast.clone().keywords.unwrap_or("Unknown".to_string()));

                tag.set_comment(&podcast_episode.description);
                let track_number = PodcastEpisodeService::get_track_number_for_episode(
                    podcast.id,
                    &podcast_episode.date_of_recording,
                );

                match track_number {
                    Ok(track_number) => {
                        tag.set_track_number(track_number as u16);
                    }
                    Err(e) => {
                        log::error!("Error getting track number: {:?}", e);
                        e.to_string();
                    }
                }

                tag.write_to_path(&paths.filename).unwrap();
                Ok(())
            }
            Err(e) => {
                log::error!("Error reading metadata: {:?}", e);
                let err: CustomError = CustomErrorInner::Conflict(e.to_string()).into();
                Err(err)
            }
        }
    }
}
