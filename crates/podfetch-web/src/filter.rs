use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct Filter {
    pub user_id: String,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}

impl Filter {
    pub fn new(
        user_id: Uuid,
        title: Option<String>,
        ascending: bool,
        filter: Option<String>,
        only_favored: bool,
    ) -> Self {
        Self {
            user_id: user_id.to_string(),
            title,
            ascending,
            filter,
            only_favored,
        }
    }
}

pub fn default_podcast_filter() -> Filter {
    Filter {
        user_id: Uuid::nil().to_string(),
        title: None,
        ascending: true,
        filter: Some("PUBLISHEDDATE".to_string()),
        only_favored: false,
    }
}

impl From<podfetch_domain::filter::Filter> for Filter {
    fn from(value: podfetch_domain::filter::Filter) -> Self {
        Self {
            user_id: value.user_id.to_string(),
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
            user_id: Uuid::parse_str(&value.user_id).unwrap_or_else(|_| Uuid::nil()),
            title: value.title,
            ascending: value.ascending,
            filter: value.filter,
            only_favored: value.only_favored,
        }
    }
}
