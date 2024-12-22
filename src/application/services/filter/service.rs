use crate::adapters::persistence::repositories::settings::filters::FilterRepository;
use crate::utils::error::CustomError;

pub struct FilterService;

impl FilterService {
    pub fn save_decision_for_timeline(username_to_search: &str, only_favored_to_insert: bool) -> Result<(), CustomError> {
        FilterRepository::save_decision_for_timeline(username_to_search, only_favored_to_insert)
    }
}