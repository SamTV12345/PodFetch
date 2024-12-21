use std::fs::File;
use crate::service::file_service::FileService;

use reqwest::blocking::ClientBuilder;

use id3::{ErrorKind, Tag, TagLike, Version};
use reqwest::header::{HeaderMap, HeaderValue};
use std::io;
use std::io::Read;
use file_format::FileFormat;
use crate::adapters::api::controllers::controller_utils::get_default_image;
use crate::adapters::filesystem::file_path::{FilenameBuilder, FilenameBuilderReturn};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::application::services::podcast_episode::service::PodcastEpisodeService;
use crate::application::services::podcast_setting::service::PodcastSettingService;
use crate::constants::inner_constants::{COMMON_USER_AGENT, DEFAULT_IMAGE_URL, PODCAST_FILENAME, PODCAST_IMAGENAME};
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::service::settings_service::SettingsService;
use crate::utils::append_to_header::add_basic_auth_headers_conditionally;
use crate::utils::error::{map_io_error, map_io_extra_error, CustomError};
use crate::utils::file_extension_determination::{determine_file_extension, FileType};

pub struct DownloadService {
    pub file_service: FileService,
}

impl DownloadService {
    pub fn new() -> Self {
        DownloadService {
            file_service: FileService::new(),
        }
    }

    pub fn download_podcast_episode(
        &mut self,
        mut podcast_episode: PodcastEpisode,
        podcast: &Podcast
    ) -> Result<(), CustomError> {
        let client = ClientBuilder::new().build().unwrap();
        let conn = &mut get_connection();
        let suffix = determine_file_extension(&podcast_episode.url, &client, FileType::Audio);
        let settings_in_db = SettingsService::get_settings()?.unwrap();
        let image_suffix =
            determine_file_extension(&podcast_episode.image_url, &client, FileType::Image);

        let mut header_map = HeaderMap::new();
        header_map.insert("User-Agent", HeaderValue::from_str(COMMON_USER_AGENT).unwrap());
        add_basic_auth_headers_conditionally(podcast_episode.url.clone(), &mut header_map);
        let mut resp = client
            .get(podcast_episode.url.clone())
            .headers(header_map.clone())
            .send()
            .unwrap();

        let mut image_response;
        match podcast_episode.image_url == DEFAULT_IMAGE_URL {
            true => {
                image_response = client
                    .get(get_default_image())
                    .headers(header_map)
                    .send()
                    .unwrap();
            }
            false => {
                let err = client
                    .get(podcast_episode.image_url.clone())
                    .headers(header_map.clone())
                    .send();
                match err {
                    Ok(response) => {
                        image_response = response;
                    }
                    Err(e) => {
                        log::error!("Error downloading image: {}", e);
                        image_response = client
                            .get(get_default_image())
                            .headers(header_map)
                            .send()
                            .unwrap();
                    }
                }
            }
        }

        let paths = match settings_in_db.use_existing_filename {
            true => FilenameBuilder::default()
                .with_podcast(podcast.clone())
                .with_suffix(&suffix)
                .with_settings(settings_in_db)
                .with_episode(podcast_episode.clone())?
                .with_filename(PODCAST_FILENAME)
                .with_image_filename(PODCAST_IMAGENAME)
                .with_image_suffix(&image_suffix)
                .with_raw_directory()?
                .build(conn)?,
            false => FilenameBuilder::default()
                .with_suffix(&suffix)
                .with_settings(settings_in_db)
                .with_image_suffix(&image_suffix)
                .with_episode(podcast_episode.clone())?
                .with_podcast_directory(&podcast.directory_name)
                .with_podcast(podcast.clone())
                .with_image_filename(PODCAST_IMAGENAME)
                .with_filename(PODCAST_FILENAME)
                .build(conn)?,
        };

        let mut podcast_out = File::create(&paths.filename).unwrap();
        let mut image_out = File::create(&paths.image_filename).unwrap();

        if !self
            .file_service
            .check_if_podcast_main_image_downloaded(&podcast.clone().directory_id)
        {
            let mut image_podcast = File::create(&paths.image_filename).unwrap();
            io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
        }

        io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");


        podcast_episode.local_image_url = paths.local_image_url.clone();
        podcast_episode.local_url = paths.local_file_url.clone();
        podcast_episode.file_image_path = Some(paths.image_filename.clone());
        podcast_episode.file_episode_path = Some(paths.filename.clone());
        PodcastEpisodeService::update_podcast_episode(&mut podcast_episode)?;

        io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
        let result = Self::handle_metadata_insertion(&paths, &mut podcast_episode, &podcast);
        if let Err(err) = result {
            log::error!("Error handling metadata insertion: {:?}", err);
        }
        Ok(())
    }

    pub fn handle_metadata_insertion(paths: &FilenameBuilderReturn, podcast_episode:
    &mut PodcastEpisode, podcast: &Podcast) -> Result<(), CustomError> {
        let detected_file = FileFormat::from_file(&paths.filename).unwrap();

        match detected_file {
            FileFormat::Mpeg12AudioLayer3 => {
                let result_of_update = Self::update_meta_data_mp3(paths, podcast_episode, podcast);
                if let Some(err) = result_of_update.err() {
                    log::error!("Error updating metadata: {:?}", err);
                }
            },
            FileFormat::AppleItunesAudio =>{
                let result_of_itunes = Self::update_meta_data_mp4(paths, podcast_episode,
                                                                  podcast);
                if let Some(err) = result_of_itunes.err() {
                    log::error!("Error updating metadata: {:?}", err);
                }
            },
            _ => {
                log::error!("File format not supported: {:?}", detected_file);
                return Err(CustomError::Conflict("File format not supported".to_string()))
            }
        }
        Ok(())
    }


    fn update_meta_data_mp3(
        paths: &FilenameBuilderReturn,
        podcast_episode: &mut PodcastEpisode,
        podcast: &Podcast
    ) -> Result<(), CustomError> {
        let mut tag = match Tag::read_from_path(&paths.filename) {
            Ok(tag) => tag,
            Err(id3::Error {
                kind: ErrorKind::NoTag,
                ..
            }) => Tag::new(),
            Err(err) => return Err(CustomError::Conflict(err.to_string())),
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
        let index = PodcastEpisodeService::get_position_of_episode(&podcast_episode.date_of_recording,
                                                          podcast_episode.podcast_id)?;

        let settings_for_podcast = PodcastSettingService::get_settings_of_podcast(podcast.id)?;

        if let Some(settings_for_podcast) = settings_for_podcast {
            if settings_for_podcast.episode_numbering {
                if  !podcast_episode.episode_numbering_processed {
                    tag.set_title(format!("{} - {}", index, &podcast_episode.name));
                    podcast_episode.episode_numbering_processed = true;
                    PodcastEpisodeService::update_podcast_episode(&podcast_episode)?;
                }
            } else {
                tag.set_title(&podcast_episode.name);
                podcast_episode.episode_numbering_processed = false;
                PodcastEpisodeService::update_podcast_episode(&podcast_episode)?;

            }
        } else {
            tag.set_title(&podcast_episode.name);
            podcast_episode.episode_numbering_processed = false;
            PodcastEpisodeService::update_podcast_episode(&podcast_episode)?;

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

        let track_number = PodcastEpisodeService::get_position_of_episode(
            &podcast_episode.date_of_recording,
            podcast.id,
        );

        if tag.track().is_none() {
            if let Ok(track_number) = track_number {
                tag.set_track(track_number as u32);
            }
        }

        let write_succesful = tag.write_to_path(&paths.filename, Version::Id3v24)
            .map(|_| ())
            .map_err(|e| CustomError::Conflict(e.to_string()));

        if write_succesful.is_err() {
           log::error!("Error writing metadata: {:?}", write_succesful.err().unwrap());
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
                let track_number = PodcastEpisodeService::get_position_of_episode(
                    &podcast_episode.date_of_recording,
                    podcast.id,
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
            },
            Err(e) => {
                log::error!("Error reading metadata: {:?}", e);
                let err = CustomError::Conflict(e.to_string());
                Err(err)
            }
        }
    }
}
