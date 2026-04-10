#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub username: String,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}

impl Filter {
    pub fn new(
        username: String,
        title: Option<String>,
        ascending: bool,
        filter: Option<String>,
        only_favored: bool,
    ) -> Self {
        Self {
            username,
            title,
            ascending,
            filter,
            only_favored,
        }
    }
}

pub trait FilterRepository: Send + Sync {
    type Error;

    fn get_by_username(&self, username: &str) -> Result<Option<Filter>, Self::Error>;
    fn save(&self, filter: Filter) -> Result<(), Self::Error>;
    fn save_timeline_decision(&self, username: &str, only_favored: bool)
    -> Result<(), Self::Error>;
}
