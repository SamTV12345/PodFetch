use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::file_service::FileService;
use std::fs::File;

use crate::adapters::file::file_handle_wrapper::FileHandleWrapper;
use crate::adapters::file::file_handler::{FileHandlerType, FileRequest};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::constants::inner_constants::{
    COMMON_USER_AGENT, ENVIRONMENT_SERVICE, PODCAST_FILENAME, PODCAST_IMAGENAME,
};
use crate::models::file_path::{FilenameBuilder, FilenameBuilderReturn};
use crate::models::podcast_episode_chapter::PodcastEpisodeChapter;
use crate::models::podcast_settings::PodcastSetting;
use crate::models::settings::Setting;
use crate::service::podcast_chapter::{Chapter, Link};
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::utils::error::{CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error};
use crate::utils::file_extension_determination::{
    DetermineFileExtensionReturn, FileType, determine_file_extension,
};
use crate::utils::http_client::get_async_sync_client;
use crate::utils::reqwest_client::get_sync_client;
use chrono::Duration;
use file_format::FileFormat;
use id3::{ErrorKind, Tag, TagLike};
use reqwest::header::{ACCEPT_ENCODING, HeaderMap, HeaderValue, USER_AGENT};
use std::io::Read;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration as StdDuration;

pub struct DownloadService {}

impl DownloadService {
    fn build_binary_sync_client() -> Result<reqwest::blocking::Client, CustomError> {
        get_sync_client()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)
    }

    async fn build_binary_async_client() -> Result<reqwest::Client, CustomError> {
        get_async_sync_client()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)
    }

    fn fetch_bytes_with_retries(url: &str) -> Result<Vec<u8>, CustomError> {
        const MAX_RETRIES: usize = 3;
        for attempt in 1..=MAX_RETRIES {
            let result = Self::build_binary_sync_client()?
                .get(url)
                .header(ACCEPT_ENCODING, "identity")
                .send()
                .map_err(map_reqwest_error)
                .and_then(|resp| resp.bytes().map_err(map_reqwest_error))
                .map(|bytes| bytes.as_ref().to_vec());

            match result {
                Ok(bytes) => return Ok(bytes),
                Err(err) if attempt == MAX_RETRIES => return Err(err),
                Err(err) => {
                    log::warn!(
                        "Download attempt {}/{} failed for {}: {}",
                        attempt,
                        MAX_RETRIES,
                        url,
                        err
                    );
                    sleep(StdDuration::from_millis(250 * attempt as u64));
                }
            }
        }
        Err(CustomErrorInner::Conflict(
            "Download failed after retries".to_string(),
            ErrorSeverity::Error,
        )
        .into())
    }

    async fn fetch_bytes_with_retries_async(url: &str) -> Result<Vec<u8>, CustomError> {
        const MAX_RETRIES: usize = 3;
        for attempt in 1..=MAX_RETRIES {
            let client = Self::build_binary_async_client().await?;
            let result = match client
                .get(url)
                .header(ACCEPT_ENCODING, "identity")
                .send()
                .await
            {
                Ok(resp) => resp
                    .bytes()
                    .await
                    .map(|bytes| bytes.as_ref().to_vec())
                    .map_err(map_reqwest_error),
                Err(err) => Err(map_reqwest_error(err)),
            };

            match result {
                Ok(bytes) => return Ok(bytes),
                Err(err) if attempt == MAX_RETRIES => return Err(err),
                Err(err) => {
                    log::warn!(
                        "Async download attempt {}/{} failed for {}: {}",
                        attempt,
                        MAX_RETRIES,
                        url,
                        err
                    );
                    tokio::time::sleep(StdDuration::from_millis(250 * attempt as u64)).await;
                }
            }
        }
        Err(CustomErrorInner::Conflict(
            "Download failed after retries".to_string(),
            ErrorSeverity::Error,
        )
        .into())
    }

    pub fn handle_suffix_response(
        dt: DetermineFileExtensionReturn,
        podcast_episode_url: &str,
    ) -> Result<(String, Vec<u8>), CustomError> {
        match dt {
            DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
            DetermineFileExtensionReturn::String(suffix) => {
                let resp = Self::fetch_bytes_with_retries(podcast_episode_url)?;
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
                let resp = Self::fetch_bytes_with_retries_async(podcast_episode_url).await?;
                Ok((suffix, resp))
            }
        }
    }

    pub fn download_podcast_episode(
        podcast_episode: PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<(), CustomError> {
        let mut header_map = HeaderMap::new();
        header_map.insert(USER_AGENT, HeaderValue::from_static(COMMON_USER_AGENT));
        header_map.insert(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
        let client = get_sync_client()
            .default_headers(header_map)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)?;
        let conn = &mut get_connection();
        let mut podcast_data = Self::handle_suffix_response(
            determine_file_extension(&podcast_episode.url, &client, FileType::Audio),
            &podcast_episode.url,
        )?;
        let settings_in_db = Setting::get_settings()?.unwrap();
        let should_download_main_image =
            !FileService::check_if_podcast_main_image_downloaded(&podcast.clone().directory_id, conn);

        let mut image_data = if should_download_main_image {
            Some(Self::handle_suffix_response(
                determine_file_extension(&podcast_episode.image_url, &client, FileType::Image),
                &podcast_episode.image_url,
            )?)
        } else {
            None
        };

        let paths = match settings_in_db.use_existing_filename {
            true => FilenameBuilder::default()
                .with_podcast(podcast.clone())
                .with_suffix(&podcast_data.0)
                .with_settings(settings_in_db)
                .with_episode(podcast_episode.clone())?
                .with_filename(PODCAST_FILENAME)
                .with_image_filename(PODCAST_IMAGENAME)
                .with_image_suffix(
                    &image_data
                        .as_ref()
                        .map(|data| data.0.clone())
                        .unwrap_or_else(|| "jpg".to_string()),
                )
                .with_raw_directory()?
                .build(conn)?,
            false => FilenameBuilder::default()
                .with_suffix(&podcast_data.0)
                .with_settings(settings_in_db)
                .with_image_suffix(
                    &image_data
                        .as_ref()
                        .map(|data| data.0.clone())
                        .unwrap_or_else(|| "jpg".to_string()),
                )
                .with_episode(podcast_episode.clone())?
                .with_podcast_directory(&podcast.directory_name)
                .with_podcast(podcast.clone())
                .with_image_filename(PODCAST_IMAGENAME)
                .with_filename(PODCAST_FILENAME)
                .build(conn)?,
        };

        if !FileHandleWrapper::path_exists(
            &podcast.directory_name,
            FileRequest::Directory,
            &ENVIRONMENT_SERVICE.default_file_handler,
        ) {
            FileHandleWrapper::create_dir(
                &podcast.directory_name,
                &ENVIRONMENT_SERVICE.default_file_handler,
            )?;
        }

        if let Some(p) = PathBuf::from(&paths.filename).parent()
            && !FileHandleWrapper::path_exists(
                p.to_str().unwrap(),
                FileRequest::Directory,
                &ENVIRONMENT_SERVICE.default_file_handler,
            )
        {
            FileHandleWrapper::create_dir(
                p.to_str().unwrap(),
                &ENVIRONMENT_SERVICE.default_file_handler,
            )?;
        }

        if should_download_main_image
            && let Some(image_data) = image_data.as_mut() {
                FileHandleWrapper::write_file(
                    &paths.image_filename,
                    image_data.1.as_mut_slice(),
                    &ENVIRONMENT_SERVICE.default_file_handler,
                )?;
            }

        FileHandleWrapper::write_file(
            &paths.filename,
            podcast_data.1.as_mut_slice(),
            &ENVIRONMENT_SERVICE.default_file_handler,
        )?;

        PodcastEpisode::update_local_paths(
            &podcast_episode.episode_id,
            &paths.image_filename,
            &paths.filename,
            conn,
        )?;
        let result = Self::handle_metadata_insertion(&paths, &podcast_episode, podcast);
        if let Ok(chapters) = &result {
            log::info!("Inserting chapters for episode {}", podcast_episode.id);
            for chapter in chapters {
                let res = PodcastEpisodeChapter::save_chapter(chapter, &podcast_episode);
                if let Err(err) = res {
                    log::error!(
                        "Error while saving chapter for episode {}: {}",
                        podcast_episode.id,
                        err
                    );
                }
            }
        }

        if let Err(err) = result {
            log::error!("Error handling metadata insertion: {err:?}");
        }
        Ok(())
    }

    pub fn handle_metadata_insertion(
        paths: &FilenameBuilderReturn,
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<Vec<Chapter>, CustomError> {
        if ENVIRONMENT_SERVICE.default_file_handler == FileHandlerType::S3 {
            return Ok(vec![]);
        }

        let chapters: Vec<Chapter>;

        let detected_file = FileFormat::from_file(&paths.filename).unwrap();
        match detected_file {
            FileFormat::Mpeg12AudioLayer3
            | FileFormat::Mpeg12AudioLayer2
            | FileFormat::AppleItunesAudio
            | FileFormat::Id3v2
            | FileFormat::WaveformAudio => {
                chapters = Self::read_chapters_from_mp3(&paths.filename)?;
                let result_of_update = Self::update_meta_data_mp3(paths, podcast_episode, podcast);
                if let Some(err) = result_of_update.err() {
                    log::error!("Error updating metadata: {err:?}");
                }
            }
            FileFormat::Mpeg4Part14 | FileFormat::Mpeg4Part14Audio => {
                chapters = Self::read_chapters_from_mp4(&paths.filename);
                let result_of_update = Self::update_meta_data_mp4(paths, podcast_episode, podcast);
                if let Some(err) = result_of_update.err() {
                    log::error!("Error updating metadata: {err:?}");
                }
            }
            _ => {
                log::error!("File format not supported: {detected_file:?}");
                return Err(CustomErrorInner::Conflict(
                    "File format not supported".to_string(),
                    ErrorSeverity::Error,
                )
                .into());
            }
        }
        Ok(chapters)
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
            Err(err) => {
                return Err(
                    CustomErrorInner::Conflict(err.to_string(), ErrorSeverity::Error).into(),
                );
            }
        };

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

        if let Some(author) = &podcast.author {
            tag.set_artist(author);
        }

        tag.set_album(&podcast.name);

        tag.set_date_recorded(podcast_episode.date_of_recording.parse().unwrap());

        if tag.genres().is_none()
            && let Some(keywords) = &podcast.keywords
        {
            tag.set_genre(keywords);
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

        if tag.track().is_none()
            && let Ok(track_number) = track_number
        {
            tag.set_track(track_number as u32);
        }

        let write_succesful: Result<(), CustomError> = tag
            // Always write ID3v2.4 because otherwise there are compatibility issues with
            // embedded tags
            .write_to_path(&paths.filename, id3::Version::Id3v24)
            .map(|_| ())
            .map_err(|e| CustomErrorInner::Conflict(e.to_string(), ErrorSeverity::Error).into());

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
                        log::error!("Error getting track number: {e:?}");
                        e.to_string();
                    }
                }

                tag.write_to_path(&paths.filename).unwrap();
                Ok(())
            }
            Err(e) => {
                log::error!("Error reading metadata: {e:?}");
                let err: CustomError =
                    CustomErrorInner::Conflict(e.to_string(), ErrorSeverity::Error).into();
                Err(err)
            }
        }
    }

    pub fn read_chapters_from_mp3(path: &String) -> Result<Vec<Chapter>, CustomError> {
        let tag = Tag::read_from_path(path)
            .map_err(|e| format!("Error reading ID3 tag from `{}`: {}", path, e));

        let tag = match tag {
            Ok(tag) => tag,
            Err(err) => {
                log::error!("Error reading ID3 tag: {}", err);
                return Ok(Vec::new());
            }
        };

        let mut chapters = Vec::new();

        for id3_chapter in tag.chapters() {
            let start = Duration::milliseconds(id3_chapter.start_time as i64);

            let temp_end = Duration::milliseconds(id3_chapter.end_time as i64);
            // Some programs might encode chapters as instants, i.e., with the start and end time being the same.
            let end = if temp_end == start {
                None
            } else {
                Some(temp_end)
            };

            let mut title = None;
            let mut link = None;

            for subframe in &id3_chapter.frames {
                match subframe.content() {
                    id3::Content::Text(text) => {
                        title = Some(text.clone());
                    }
                    id3::Content::Link(url) => {
                        link = Some(Link {
                            url: url::Url::parse(url)
                                .map_err(<url::ParseError as Into<CustomError>>::into)?,
                            title: None,
                        });
                    }
                    id3::Content::ExtendedLink(extended_link) => {
                        link = Some(Link {
                            url: url::Url::parse(&extended_link.link)
                                .map_err(<url::ParseError as Into<CustomError>>::into)?,
                            title: match extended_link.description.trim() {
                                "" => None,
                                description => Some(description.to_string()),
                            },
                        });
                    }
                    _ => {}
                }
            }

            chapters.push(Chapter {
                title,
                link,
                start,
                end,
                ..Default::default()
            });
        }

        // Order chapters by start time.
        chapters.sort_by(|a, b| a.start.cmp(&b.start));

        Ok(chapters)
    }

    pub fn read_chapters_from_mp4(path: &String) -> Vec<Chapter> {
        let tag = mp4ameta::Tag::read_from_path(path);
        let tag = match tag {
            Ok(tag) => tag,
            Err(err) => {
                log::error!("Error reading MP4 tag: {}", err);
                return Vec::new();
            }
        };
        let chapters_list: Vec<_> = tag.chapter_list().iter().collect();
        let mut chapters = Vec::new();
        for (index, id3_chapter) in chapters_list.iter().enumerate() {
            let start = Duration::milliseconds(id3_chapter.start.as_millis() as i64);

            let end = if index + 1 < chapters_list.len() {
                Some(Duration::milliseconds(
                    chapters_list[index + 1].start.as_millis() as i64,
                ))
            } else {
                None
            };

            let title = if id3_chapter.title.is_empty() {
                None
            } else {
                Some(id3_chapter.title.clone())
            };

            chapters.push(Chapter {
                title,
                link: None,
                start,
                end,
                ..Default::default()
            });
        }
        chapters
    }
}
