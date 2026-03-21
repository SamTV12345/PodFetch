use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
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

impl From<podfetch_domain::filter::Filter> for Filter {
    fn from(value: podfetch_domain::filter::Filter) -> Self {
        Self {
            username: value.username,
            title: value.title,
            ascending: value.ascending,
            filter: value.filter,
            only_favored: value.only_favored,
        }
    }
}

impl From<Filter> for podfetch_domain::filter::Filter {
    fn from(value: Filter) -> Self {
        Self {
            username: value.username,
            title: value.title,
            ascending: value.ascending,
            filter: value.filter,
            only_favored: value.only_favored,
        }
    }
}
