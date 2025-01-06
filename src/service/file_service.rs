use std::collections::HashMap;

use crate::models::podcasts::Podcast;
use std::io::{Error, Write};

use std::path::Path;
use std::str::FromStr;

use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::controllers::settings_controller::ReplacementStrategy;
use crate::models::misc_models::PodcastInsertModel;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_settings::PodcastSetting;
use crate::models::settings::Setting;
use crate::service::download_service::DownloadService;
use crate::service::path_service::PathService;
use crate::service::settings_service::SettingsService;
use crate::utils::error::{map_io_error, CustomError};
use crate::utils::file_extension_determination::{determine_file_extension, FileType};
use crate::utils::file_name_replacement::{Options, Sanitizer};
use crate::utils::rss_feed_parser::RSSFeedParser;
use crate::DBType as DbConnection;
use regex::Regex;
use rss::Channel;
use tokio::task::spawn_blocking;

#[derive(Clone)]
pub struct FileService {}

impl FileService {
    pub fn check_if_podcast_main_image_downloaded(
        podcast_id: &str,
        conn: &mut DbConnection,
    ) -> bool {
        let podcast = Podcast::get_podcast_by_directory_id(podcast_id, conn).unwrap();
        match podcast {
            Some(podcast) => {
                if !podcast.image_url.contains("http") {
                    return Path::new(&podcast.image_url).exists();
                }
            }
            None => {
                return false;
            }
        }
        false
    }

    pub fn create_podcast_root_directory_exists() -> Result<(), Error> {
        if !Path::new("podcasts").exists() {
            return std::fs::create_dir("podcasts");
        }

        Ok(())
    }

    pub async fn create_podcast_directory_exists(
        podcast_insert_model: &PodcastInsertModel,
        channel: Option<Channel>,
    ) -> Result<String, CustomError> {
        let escaped_title =
            prepare_podcast_title_to_directory(podcast_insert_model, channel).await?;
        let escaped_path = format!("podcasts/{}", escaped_title);
        if !Path::new(&escaped_path).exists() {
            std::fs::create_dir(escaped_path.clone())
                .map_err(|err| map_io_error(err, Some(escaped_path.clone())))?;
            Ok(escaped_path)
        } else {
            // Check if this is a new podcast with the same name as an old one

            let conn = &mut get_connection();
            let podcast =
                Podcast::get_podcast_by_directory_id(&podcast_insert_model.id.to_string(), conn)?;
            match podcast {
                Some(_) => {
                    // is the same podcast
                    Ok(escaped_path)
                }
                None => {
                    // has not been inserted into the database yet
                    let mut i = 1;
                    while Path::new(&format!("podcasts/{}-{}", escaped_title, i)).exists() {
                        i += 1;
                    }
                    // This is save to insert because this directory does not exist
                    std::fs::create_dir(format!("podcasts/{}-{}", escaped_title, i)).map_err(
                        |err| map_io_error(err, Some(format!("podcasts/{}-{}", escaped_title, i))),
                    )?;
                    Ok(format!("podcasts/{}-{}", escaped_title, i))
                }
            }
        }
    }

    pub async fn download_podcast_image(
        podcast_path: &str,
        image_url: String,
        podcast_id: &str,
    ) -> Result<(), CustomError> {
        let image_url_cloned = image_url.clone();
        let mut image_suffix = DownloadService::handle_suffix_response_async(
            spawn_blocking(move || {
                let client = reqwest::blocking::Client::new();
                determine_file_extension(&image_url_cloned, &client, FileType::Image)
            })
            .await
            .unwrap(),
            &image_url,
        )
        .await?;

        let file_path =
            PathService::get_image_podcast_path_with_podcast_prefix(podcast_path, &image_suffix.0);
        let mut image_out = std::fs::File::create(file_path.0.clone()).unwrap();
        image_out.write_all(image_suffix.1.as_mut_slice()).unwrap();
        PodcastEpisode::update_podcast_image(podcast_id, &file_path.1)?;
        Ok(())
    }

    pub fn cleanup_old_episode(episode: &PodcastEpisode) -> Result<(), CustomError> {
        log::info!("Cleaning up old episode: {}", episode.episode_id);

        fn check_if_file_exists(file_path: &str) -> bool {
            std::fs::exists(file_path).unwrap()
        }
        if let Some(episode_path) = episode.file_episode_path.clone() {
            if check_if_file_exists(&episode_path) {
                std::fs::remove_file(episode_path)
                    .map_err(|e| map_io_error(e, episode.file_episode_path.clone()))?;
            }
        }
        if let Some(image_path) = episode.file_image_path.clone() {
            if check_if_file_exists(&image_path) {
                std::fs::remove_file(image_path)
                    .map_err(|e| map_io_error(e, episode.file_image_path.clone()))?;
            }
        }
        Ok(())
    }

    pub fn delete_podcast_files(podcast_dir: &str) {
        std::fs::remove_dir_all(podcast_dir).expect("Error deleting podcast directory");
    }
}

pub async fn prepare_podcast_title_to_directory(
    podcast: &PodcastInsertModel,
    channel: Option<Channel>,
) -> Result<String, CustomError> {
    let retrieved_settings = SettingsService::get_settings()?.unwrap();
    let opt_podcast_settings = PodcastSetting::get_settings(podcast.id)?;

    let podcast = match channel {
        Some(channel) => RSSFeedParser::parse_rss_feed(channel),
        None => {
            let client = reqwest::Client::new();
            let rss_feed = podcast.feed_url.clone();
            let feed_response = client.get(rss_feed).send().await.unwrap();
            let content = feed_response.bytes().await.unwrap();

            let channel = Channel::read_from(&content[..]);
            RSSFeedParser::parse_rss_feed(channel.unwrap())
        }
    };

    perform_podcast_variable_replacement(retrieved_settings, podcast.clone(), opt_podcast_settings)
}

fn replace_date_of_str(date: &str) -> String {
    match date.contains('T') {
        true => {
            let splitted_date = date.split('T').collect::<Vec<&str>>();
            splitted_date[0].to_string()
        }
        false => date.to_string(),
    }
}

pub fn perform_podcast_variable_replacement(
    retrieved_settings: Setting,
    podcast: crate::utils::rss_feed_parser::PodcastParsed,
    podcast_setting: Option<PodcastSetting>,
) -> Result<String, CustomError> {
    let sanitizer = Sanitizer::new(None);
    let escaped_podcast_title = perform_replacement(
        &podcast.title,
        retrieved_settings.clone(),
        podcast_setting.clone(),
    )
    .replace(|c: char| !c.is_ascii(), "");
    let podcast_format;

    if podcast_setting.is_none() {
        podcast_format = retrieved_settings.podcast_format.clone();
    } else if let Some(e) = &podcast_setting {
        if e.activated {
            podcast_format = e.podcast_format.clone();
        } else {
            podcast_format = retrieved_settings.podcast_format.clone();
        }
    } else {
        podcast_format = retrieved_settings.podcast_format.clone();
    }

    if podcast_format.is_empty() || podcast_format.trim() == "{}" {
        return Ok(sanitizer.sanitize(podcast.title));
    }

    let mut vars: HashMap<String, &str> = HashMap::new();

    let podcast_summary = podcast.summary;
    let podcast_language = podcast.language;
    let podcast_explicit = podcast.explicit;
    let podcast_keyword = podcast.keywords;
    let podcast_date = replace_date_of_str(&podcast.date);

    // Insert variables
    vars.insert("podcastTitle".to_string(), &escaped_podcast_title);
    vars.insert("podcastDescription".to_string(), &podcast_summary);
    vars.insert("podcastLanguage".to_string(), &podcast_language);
    vars.insert("podcastExplicit".to_string(), &podcast_explicit);
    vars.insert("podcastKeywords".to_string(), &podcast_keyword);
    vars.insert("date".to_string(), &podcast_date);

    let fixed_string = podcast_format
        .replace("{title}", "{podcastTitle}")
        .replace("{description}", "{podcastDescription}")
        .replace("{language}", "{podcastLanguage}")
        .replace("{explicit}", "{podcastExplicit}")
        .replace("{keywords}", "{podcastKeywords}")
        .chars()
        .filter(|&c| c as u32 != 44)
        .collect::<String>();

    let result = strfmt::strfmt(fixed_string.trim(), &vars);

    match result {
        Ok(res) => Ok(sanitizer.sanitize(res)),
        Err(err) => {
            log::error!("Error formatting podcast title: {}", err);
            Err(CustomError::Conflict(err.to_string()))
        }
    }
}

pub fn prepare_podcast_episode_title_to_directory(
    podcast_episode: PodcastEpisode,
) -> Result<String, CustomError> {
    let retrieved_settings = SettingsService::get_settings()?.unwrap();
    if retrieved_settings.use_existing_filename {
        let res_of_filename = get_filename_of_url(&podcast_episode.url);
        if let Ok(res_unwrapped) = res_of_filename {
            return Ok(res_unwrapped);
        }
    }
    let podcast_settings = PodcastSetting::get_settings(podcast_episode.podcast_id)?;
    perform_episode_variable_replacement(retrieved_settings, podcast_episode, podcast_settings)
}

pub fn perform_episode_variable_replacement(
    retrieved_settings: Setting,
    podcast_episode: PodcastEpisode,
    podcast_settings: Option<PodcastSetting>,
) -> Result<String, CustomError> {
    let escaped_episode_title = perform_replacement(
        &podcast_episode.name,
        retrieved_settings.clone(),
        podcast_settings.clone(),
    )
    .replace(|c: char| !c.is_ascii(), "");
    let episode_format;

    if podcast_settings.is_none() {
        episode_format = retrieved_settings.episode_format.clone();
    } else if let Some(e) = &podcast_settings {
        if e.activated {
            episode_format = e.episode_format.clone();
        } else {
            episode_format = retrieved_settings.episode_format.clone();
        }
    } else {
        episode_format = retrieved_settings.episode_format.clone();
    }

    if episode_format.is_empty() || episode_format.trim() == "{}" {
        return Ok(escaped_episode_title);
    }

    let mut vars: HashMap<String, &str> = HashMap::new();

    let total_time = podcast_episode.total_time.to_string();
    let episode_date = replace_date_of_str(&podcast_episode.date_of_recording);
    // Insert variables
    vars.insert("episodeTitle".to_string(), &escaped_episode_title);
    vars.insert("episodeDate".to_string(), &episode_date);
    vars.insert("episodeGuid".to_string(), &podcast_episode.guid);
    vars.insert("episodeUrl".to_string(), &podcast_episode.url);
    vars.insert(
        "episodeDescription".to_string(),
        &podcast_episode.description,
    );
    vars.insert("episodeDuration".to_string(), &total_time);

    let fixed_string = episode_format
        .replace("{title}", "{episodeTitle}")
        .replace("{date}", "{episodeDate}")
        .replace("{description}", "{episodeDescription}")
        .replace("{duration}", "{episodeDuration}")
        .replace("{guid}", "{episodeGuid}")
        .replace("{url}", "{episodeUrl}")
        .chars()
        .filter(|&c| c as u32 != 44)
        .collect::<String>();

    let result = strfmt::strfmt(fixed_string.trim(), &vars);

    match result {
        Ok(res) => Ok(res.to_string()),
        Err(err) => {
            log::error!("Error formatting episode title: {}", err);
            Err(CustomError::Conflict(err.to_string()))
        }
    }
}

fn perform_replacement(
    title: &str,
    retrieved_settings: Setting,
    podcast_settings: Option<PodcastSetting>,
) -> String {
    let mut final_string: String = title.to_string();
    let replacement_strategy;
    if podcast_settings.is_none() {
        replacement_strategy = retrieved_settings.replacement_strategy.clone();
    } else if let Some(e) = &podcast_settings {
        if e.activated {
            replacement_strategy = e.replacement_strategy.clone();
        } else {
            replacement_strategy = retrieved_settings.replacement_strategy.clone();
        }
    } else {
        replacement_strategy = retrieved_settings.replacement_strategy.clone();
    }

    // Colon replacement strategy
    match ReplacementStrategy::from_str(&replacement_strategy).unwrap() {
        ReplacementStrategy::ReplaceWithDashAndUnderscore => {
            let sanitizer = Sanitizer::new(Some(Options::default_with_replacement("-_")));
            final_string = sanitizer.sanitize(&final_string);
        }
        ReplacementStrategy::Remove => {
            let sanitizer = Sanitizer::new(Some(Options::default_with_replacement("")));
            final_string = sanitizer.sanitize(&final_string);
        }
        ReplacementStrategy::ReplaceWithDash => {
            let sanitizer = Sanitizer::new(Some(Options::default_with_replacement("-")));
            final_string = sanitizer.sanitize(&final_string);
        }
    }
    final_string
}

fn get_filename_of_url(url: &str) -> Result<String, String> {
    let re = Regex::new(r"/([^/?]+)\.\w+(?:\?.*)?$").unwrap();

    if let Some(captures) = re.captures(url) {
        let dir_name = remove_extension(captures.get(1).unwrap().as_str()).to_string();

        return Ok(dir_name);
    }
    Err("Could not get filename".to_string())
}

fn remove_extension(filename: &str) -> &str {
    if let Some(dot_idx) = filename.rfind('.') {
        &filename[..dot_idx]
    } else {
        filename
    }
}

#[cfg(test)]
mod tests {
    use crate::models::podcast_episode::PodcastEpisode;
    use crate::models::settings::Setting;
    use crate::service::file_service::{
        perform_episode_variable_replacement, perform_podcast_variable_replacement,
        perform_replacement,
    };
    use crate::utils::rss_feed_parser::PodcastParsed;

    #[test]
    fn test_remove_file_suffix() {
        let filename = "test.mp3";
        let filename_without_suffix = super::remove_extension(filename);
        assert_eq!(filename_without_suffix, "test");
    }

    #[test]
    fn test_remove_file_suffix_long_name() {
        let filename = "testz398459345z!?234.mp3";
        let filename_without_suffix = super::remove_extension(filename);
        assert_eq!(filename_without_suffix, "testz398459345z!?234");
    }

    #[test]
    fn get_filename_of_url_test() {
        let url = "https://www.example.com/test.mp3";
        let filename = super::get_filename_of_url(url);
        assert_eq!(filename.unwrap(), "test");
    }

    #[test]
    fn get_filename_of_url_test_with_numbers() {
        let url = "https://www.example823459274892347.com234/mypodcast.mp3";
        let filename = super::get_filename_of_url(url);
        assert_eq!(filename.unwrap(), "mypodcast");
    }

    #[test]
    fn test_perform_replacement_dash_and_underscore() {
        let title = "test: test";
        let settings = Setting {
            id: 1,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{podcasttitle}".to_string(),
            episode_format: "test123{episodetitle}".to_string(),
            replacement_strategy: "replace-with-dash-and-underscore".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,

            podcast_prefill: 0,
            direct_paths: false,
        };

        let result = perform_replacement(title, settings, None);

        assert_eq!(result, "test-_ test");
    }

    #[test]
    fn test_perform_replacement_remove() {
        let title = "test: test";
        let settings = Setting {
            id: 1,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{podcasttitle}".to_string(),
            episode_format: "test123{episodetitle}".to_string(),
            replacement_strategy: "remove".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,

            podcast_prefill: 0,
            direct_paths: false,
        };

        let result = perform_replacement(title, settings, None);

        assert_eq!(result, "test test");
    }

    #[test]
    fn test_perform_replacement_replace_with_dash() {
        let title = "test: test";
        let settings = Setting {
            id: 1,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{podcasttitle}".to_string(),
            episode_format: "test123{episodetitle}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,

            podcast_prefill: 0,
            direct_paths: false,
        };

        let result = perform_replacement(title, settings, None);

        assert_eq!(result, "test- test");
    }

    #[test]
    fn test_podcast_episode_replacement_guid() {
        let settings = Setting {
            id: 2,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "test{guid}".to_string(),
            episode_format: "test123{guid}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,
            podcast_prefill: 0,
            direct_paths: false,
        };

        let podcast_episode = PodcastEpisode {
            id: 2,
            name: "test".to_string(),
            description: "test".to_string(),
            status: "".to_string(),
            url: "test".to_string(),
            guid: "test".to_string(),
            total_time: 0,
            local_url: "".to_string(),
            date_of_recording: "2022".to_string(),
            podcast_id: 0,
            file_episode_path: None,
            file_image_path: None,
            episode_id: "".to_string(),
            image_url: "".to_string(),
            local_image_url: "".to_string(),
            download_time: None,
            deleted: false,
            episode_numbering_processed: false,
        };

        let result = perform_episode_variable_replacement(settings, podcast_episode, None);
        assert_eq!(result.unwrap(), "test123test");
    }

    #[test]
    fn test_podcast_episode_replacement_title() {
        let settings = Setting {
            id: 2,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "{date}{title}".to_string(),
            episode_format: "{date}{title}{guid}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,
            podcast_prefill: 0,
            direct_paths: false,
        };

        let podcast_episode = PodcastEpisode {
            id: 2,
            name: "MyPodcast".to_string(),
            description: "test".to_string(),
            status: "".to_string(),
            url: "test".to_string(),
            guid: "test".to_string(),
            total_time: 0,
            local_url: "".to_string(),
            date_of_recording: "2022".to_string(),
            podcast_id: 0,
            file_episode_path: None,
            file_image_path: None,
            episode_id: "".to_string(),
            image_url: "".to_string(),
            local_image_url: "".to_string(),
            download_time: None,
            deleted: false,
            episode_numbering_processed: false,
        };

        let result = perform_episode_variable_replacement(settings, podcast_episode, None);
        assert_eq!(result.unwrap(), "2022MyPodcasttest");
    }

    #[test]
    fn test_podcast_episode_replacement_old_format() {
        let settings = Setting {
            id: 2,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "{date}{title}".to_string(),
            episode_format: "{}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,
            podcast_prefill: 0,
            direct_paths: false,
        };

        let podcast_episode = PodcastEpisode {
            id: 2,
            name: "MyPodcast".to_string(),
            description: "test".to_string(),
            status: "".to_string(),
            url: "test2".to_string(),
            guid: "test".to_string(),
            total_time: 0,
            local_url: "".to_string(),
            date_of_recording: "2022".to_string(),
            podcast_id: 0,
            file_episode_path: None,
            file_image_path: None,
            episode_id: "".to_string(),
            image_url: "".to_string(),
            local_image_url: "".to_string(),
            download_time: None,
            deleted: false,
            episode_numbering_processed: false,
        };

        let result = perform_episode_variable_replacement(settings, podcast_episode, None);
        assert_eq!(result.unwrap(), "MyPodcast");
    }

    #[test]
    pub fn perform_podcast_variable_replacement_date_title() {
        let settings = Setting {
            id: 2,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "{date}-{title}".to_string(),
            episode_format: "{date}{}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,
            podcast_prefill: 0,
            direct_paths: false,
        };

        let podcast_episode = PodcastParsed {
            title: "Test".to_string(),
            language: "".to_string(),
            explicit: "false".to_string(),
            keywords: "test,test2".to_string(),
            summary: "test123".to_string(),
            date: "2022-12".to_string(),
        };
        let result = perform_podcast_variable_replacement(settings, podcast_episode, None);
        assert_eq!(result.unwrap(), "2022-12-Test");
    }

    #[test]
    pub fn perform_podcast_variable_replacement_old_format() {
        let settings = Setting {
            id: 2,
            auto_download: false,
            auto_update: false,
            auto_cleanup: false,
            auto_cleanup_days: 0,
            podcast_format: "{}".to_string(),
            episode_format: "{date}{title}".to_string(),
            replacement_strategy: "replace-with-dash".to_string(),
            replace_invalid_characters: true,
            use_existing_filename: false,
            podcast_prefill: 0,
            direct_paths: false,
        };

        let podcast_episode = PodcastParsed {
            title: "Test".to_string(),
            language: "en".to_string(),
            explicit: "false".to_string(),
            keywords: "test,test2".to_string(),
            summary: "test123".to_string(),
            date: "2022-12".to_string(),
        };
        let result = perform_podcast_variable_replacement(settings, podcast_episode, None);
        assert_eq!(result.unwrap(), "Test");
    }
}
