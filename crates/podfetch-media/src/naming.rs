//! Naming the podcast directories and episode files after the configured
//! schemes (`{date}-{title}`, `{episodeNumber:0>3} - {episodeTitle}`, …).

use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::rss::PodcastParsed;
use podfetch_storage::{Options, Sanitizer};
use std::collections::HashMap;
use std::sync::LazyLock;

/// How characters that are invalid in file names are replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Replacement {
    #[default]
    DashAndUnderscore,
    Remove,
    Dash,
}

impl Replacement {
    /// Reads the stored setting; an unknown value falls back to the default.
    pub fn from_setting(value: &str) -> Self {
        match value {
            "replace-with-dash-and-underscore" => Self::DashAndUnderscore,
            "remove" => Self::Remove,
            "replace-with-dash" => Self::Dash,
            _ => {
                tracing::warn!("Unknown replacement strategy '{value}', using the default");
                Self::default()
            }
        }
    }

    fn replacement(self) -> &'static str {
        match self {
            Self::DashAndUnderscore => "-_",
            Self::Remove => "",
            Self::Dash => "-",
        }
    }
}

/// The naming scheme in effect for one podcast: the podcast's own settings
/// when they are activated, the instance-wide ones otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamingRules {
    pub replacement: Replacement,
    pub podcast_format: String,
    pub episode_format: String,
}

/// What an episode file is named after, taken from the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpisodeNaming {
    pub name: String,
    pub date_of_recording: String,
    pub guid: String,
    pub url: String,
    pub description: String,
    pub total_time: i32,
}

/// Replaces the characters of `title` that can't appear in a file name.
pub fn sanitize_title(title: &str, replacement: Replacement) -> String {
    Sanitizer::new(Some(Options::default_with_replacement(
        replacement.replacement(),
    )))
    .sanitize(title)
}

/// The directory name of a podcast.
pub fn podcast_directory_name(
    rules: &NamingRules,
    podcast: &PodcastParsed,
) -> Result<String, CustomError> {
    let sanitizer = Sanitizer::new(None);
    let escaped_podcast_title = sanitize_title(&podcast.title, rules.replacement);
    let podcast_format = &rules.podcast_format;

    if podcast_format.is_empty() || podcast_format.trim() == "{}" {
        return Ok(sanitizer.sanitize(&podcast.title));
    }

    let podcast_date = replace_date_of_str(&podcast.date);
    let vars: HashMap<String, &str> = HashMap::from([
        ("podcastTitle".to_string(), escaped_podcast_title.as_str()),
        ("podcastDescription".to_string(), podcast.summary.as_str()),
        ("podcastLanguage".to_string(), podcast.language.as_str()),
        ("podcastExplicit".to_string(), podcast.explicit.as_str()),
        ("podcastKeywords".to_string(), podcast.keywords.as_str()),
        ("date".to_string(), podcast_date.as_str()),
    ]);

    let fixed_string = podcast_format
        .replace("{title}", "{podcastTitle}")
        .replace("{description}", "{podcastDescription}")
        .replace("{language}", "{podcastLanguage}")
        .replace("{explicit}", "{podcastExplicit}")
        .replace("{keywords}", "{podcastKeywords}")
        .chars()
        .filter(|&c| c != ',')
        .collect::<String>();

    match strfmt::strfmt(fixed_string.trim(), &vars) {
        Ok(res) => Ok(sanitizer.sanitize(res)),
        Err(err) => {
            tracing::error!("Error formatting podcast title: {err}");
            Err(CustomErrorInner::Conflict(err.to_string(), ErrorSeverity::Error).into())
        }
    }
}

/// The file name of an episode, without extension. `episode_number` is the
/// episode's position in the podcast; with `total_episode_count` the number
/// is zero-padded to the width of the count.
pub fn episode_file_stem(
    rules: &NamingRules,
    episode: &EpisodeNaming,
    episode_number: i64,
    total_episode_count: Option<usize>,
) -> Result<String, CustomError> {
    let escaped_episode_title = sanitize_title(&episode.name, rules.replacement);
    let episode_format = &rules.episode_format;

    if episode_format.is_empty() || episode_format.trim() == "{}" {
        return Ok(escaped_episode_title);
    }

    let total_time = episode.total_time.to_string();
    let episode_number_str = match total_episode_count {
        Some(total) if total > 0 => {
            let digits = (total as f64).log10().floor() as usize + 1;
            format!("{episode_number:0>digits$}")
        }
        _ => episode_number.to_string(),
    };
    let episode_date = replace_date_of_str(&episode.date_of_recording);
    let vars: HashMap<String, &str> = HashMap::from([
        ("episodeTitle".to_string(), escaped_episode_title.as_str()),
        ("episodeDate".to_string(), episode_date.as_str()),
        ("episodeGuid".to_string(), episode.guid.as_str()),
        ("episodeUrl".to_string(), episode.url.as_str()),
        (
            "episodeDescription".to_string(),
            episode.description.as_str(),
        ),
        ("episodeDuration".to_string(), total_time.as_str()),
        ("episodeNumber".to_string(), episode_number_str.as_str()),
    ]);

    let fixed_string = episode_format
        .replace("{title}", "{episodeTitle}")
        .replace("{date}", "{episodeDate}")
        .replace("{description}", "{episodeDescription}")
        .replace("{duration}", "{episodeDuration}")
        .replace("{guid}", "{episodeGuid}")
        .replace("{url}", "{episodeUrl}")
        .chars()
        .filter(|&c| c != ',')
        .collect::<String>();

    strfmt::strfmt(fixed_string.trim(), &vars).map_err(|err| {
        tracing::error!("Error formatting episode title: {err}");
        CustomErrorInner::Conflict(err.to_string(), ErrorSeverity::Error).into()
    })
}

/// The file name of `url` without its extension, if it has one.
pub fn filename_of_url(url: &str) -> Option<String> {
    static FILENAME: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"/([^/?]+)\.\w+(?:\?.*)?$").expect("static regex is valid")
    });
    FILENAME
        .captures(url)
        .and_then(|captures| captures.get(1))
        .map(|name| remove_extension(name.as_str()).to_owned())
}

fn remove_extension(filename: &str) -> &str {
    filename
        .rfind('.')
        .map_or(filename, |dot_idx| &filename[..dot_idx])
}

/// The date part of an RFC 3339 timestamp; other values stay as they are.
fn replace_date_of_str(date: &str) -> String {
    date.split_once('T')
        .map_or(date, |(day, _)| day)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(replacement: Replacement, podcast_format: &str, episode_format: &str) -> NamingRules {
        NamingRules {
            replacement,
            podcast_format: podcast_format.to_string(),
            episode_format: episode_format.to_string(),
        }
    }

    fn episode(name: &str, url: &str) -> EpisodeNaming {
        EpisodeNaming {
            name: name.to_string(),
            date_of_recording: "2022".to_string(),
            guid: "test".to_string(),
            url: url.to_string(),
            description: "test".to_string(),
            total_time: 0,
        }
    }

    fn podcast(language: &str) -> PodcastParsed {
        PodcastParsed {
            title: "Test".to_string(),
            language: language.to_string(),
            explicit: "false".to_string(),
            keywords: "test,test2".to_string(),
            summary: "test123".to_string(),
            date: "2022-12".to_string(),
        }
    }

    #[test]
    fn removes_the_file_suffix() {
        assert_eq!(remove_extension("test.mp3"), "test");
        assert_eq!(
            remove_extension("testz398459345z!?234.mp3"),
            "testz398459345z!?234"
        );
    }

    #[test]
    fn takes_the_file_name_of_a_url() {
        assert_eq!(
            filename_of_url("https://www.example.com/test.mp3").as_deref(),
            Some("test")
        );
        assert_eq!(
            filename_of_url("https://www.example823459274892347.com234/mypodcast.mp3").as_deref(),
            Some("mypodcast")
        );
        assert_eq!(filename_of_url("https://www.example.com/"), None);
    }

    #[test]
    fn replaces_invalid_characters_by_strategy() {
        assert_eq!(
            sanitize_title("test: test", Replacement::DashAndUnderscore),
            "test-_ test"
        );
        assert_eq!(
            sanitize_title("test: test", Replacement::Remove),
            "test test"
        );
        assert_eq!(
            sanitize_title("test: test", Replacement::Dash),
            "test- test"
        );
    }

    #[test]
    fn reads_the_stored_strategy() {
        assert_eq!(Replacement::from_setting("remove"), Replacement::Remove);
        assert_eq!(
            Replacement::from_setting("replace-with-dash"),
            Replacement::Dash
        );
        assert_eq!(
            Replacement::from_setting("replace-with-dash-and-underscore"),
            Replacement::DashAndUnderscore
        );
        assert_eq!(
            Replacement::from_setting("bogus"),
            Replacement::DashAndUnderscore
        );
    }

    #[test]
    fn names_episodes_by_guid() {
        let rules = rules(Replacement::Dash, "test{guid}", "test123{guid}");
        assert_eq!(
            episode_file_stem(&rules, &episode("test", "test"), 1, None).unwrap(),
            "test123test"
        );
    }

    #[test]
    fn names_episodes_by_date_and_title() {
        let rules = rules(Replacement::Dash, "{date}{title}", "{date}{title}{guid}");
        assert_eq!(
            episode_file_stem(&rules, &episode("MyPodcast", "test"), 1, None).unwrap(),
            "2022MyPodcasttest"
        );
    }

    #[test]
    fn the_old_episode_format_is_the_title() {
        let rules = rules(Replacement::Dash, "{date}{title}", "{}");
        assert_eq!(
            episode_file_stem(&rules, &episode("MyPodcast", "test2"), 1, None).unwrap(),
            "MyPodcast"
        );
    }

    #[test]
    fn episode_numbers_can_be_padded() {
        let rules = rules(
            Replacement::Dash,
            "{date}{title}",
            "{episodeNumber:0>3} - {episodeTitle}",
        );
        assert_eq!(
            episode_file_stem(&rules, &episode("Hello", "test"), 7, None).unwrap(),
            "007 - Hello"
        );
        let plain = NamingRules {
            episode_format: "{episodeNumber} - {episodeTitle}".to_string(),
            ..rules
        };
        assert_eq!(
            episode_file_stem(&plain, &episode("Hello", "test"), 7, Some(120)).unwrap(),
            "007 - Hello"
        );
    }

    #[test]
    fn names_podcasts_by_date_and_title() {
        let rules = rules(Replacement::Dash, "{date}-{title}", "{date}{}");
        assert_eq!(
            podcast_directory_name(&rules, &podcast("")).unwrap(),
            "2022-12-Test"
        );
    }

    #[test]
    fn the_old_podcast_format_is_the_title() {
        let rules = rules(Replacement::Dash, "{}", "{date}{title}");
        assert_eq!(
            podcast_directory_name(&rules, &podcast("en")).unwrap(),
            "Test"
        );
    }

    #[test]
    fn an_unknown_variable_is_an_error() {
        let rules = rules(Replacement::Dash, "{nope}", "{nope}");
        assert!(podcast_directory_name(&rules, &podcast("en")).is_err());
        assert!(episode_file_stem(&rules, &episode("x", "y"), 1, None).is_err());
    }
}
