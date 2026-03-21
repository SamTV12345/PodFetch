use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Setting {
    pub id: i32,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub podcast_prefill: i32,
    pub replace_invalid_characters: bool,
    pub use_existing_filename: bool,
    pub replacement_strategy: String,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
}

#[derive(Clone)]
pub struct UpdateNameSettings {
    pub use_existing_filename: bool,
    pub replace_invalid_characters: bool,
    pub replacement_strategy: ReplacementStrategy,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool,
}

impl UpdateNameSettings {
    pub fn apply_to(&self, setting: &mut Setting) {
        setting.replace_invalid_characters = self.replace_invalid_characters;
        setting.use_existing_filename = self.use_existing_filename;
        setting.direct_paths = self.direct_paths;
        setting.replacement_strategy = self.replacement_strategy.to_string();
        setting.episode_format = self.episode_format.clone();
        setting.podcast_format = self.podcast_format.clone();
    }
}

#[derive(Clone)]
pub enum ReplacementStrategy {
    ReplaceWithDashAndUnderscore,
    Remove,
    ReplaceWithDash,
}

impl Display for ReplacementStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            ReplacementStrategy::ReplaceWithDashAndUnderscore => "replace-with-dash-and-underscore",
            ReplacementStrategy::Remove => "remove",
            ReplacementStrategy::ReplaceWithDash => "replace-with-dash",
        };
        write!(f, "{value}")
    }
}

impl FromStr for ReplacementStrategy {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "replace-with-dash-and-underscore" => {
                Ok(ReplacementStrategy::ReplaceWithDashAndUnderscore)
            }
            "remove" => Ok(ReplacementStrategy::Remove),
            "replace-with-dash" => Ok(ReplacementStrategy::ReplaceWithDash),
            _ => Err(()),
        }
    }
}

pub trait SettingRepository: Send + Sync {
    type Error;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error>;
    fn update_settings(&self, setting: Setting) -> Result<Setting, Self::Error>;
    fn insert_default_settings(&self) -> Result<(), Self::Error>;
}

