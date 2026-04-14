#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub user_id: i32,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}

impl Filter {
    pub fn new(
        user_id: i32,
        title: Option<String>,
        ascending: bool,
        filter: Option<String>,
        only_favored: bool,
    ) -> Self {
        Self {
            user_id,
            title,
            ascending,
            filter,
            only_favored,
        }
    }
}

pub trait FilterRepository: Send + Sync {
    type Error;

    fn get_by_user_id(&self, user_id: i32) -> Result<Option<Filter>, Self::Error>;
    fn save(&self, filter: Filter) -> Result<(), Self::Error>;
    fn save_timeline_decision(&self, user_id: i32, only_favored: bool)
    -> Result<(), Self::Error>;
}
