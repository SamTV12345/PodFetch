use podfetch_domain::subscription::{
    Subscription, SubscriptionChangesToClient, SubscriptionModelChanges,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct SubscriptionRetrieveRequest {
    pub since: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct SubscriptionUpdateRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct SubscriptionPostResponse {
    pub timestamp: i64,
    pub update_urls: Vec<Vec<String>>,
}

pub trait SubscriptionApplicationService {
    type Error;

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionChangesToClient, Self::Error>;
    fn get_user_subscriptions(
        &self,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionModelChanges, Self::Error>;
    fn update_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        request: SubscriptionUpdateRequest,
    ) -> Result<Vec<Vec<String>>, Self::Error>;
}

pub fn to_client_changes(changes: SubscriptionModelChanges) -> SubscriptionChangesToClient {
    changes.into()
}

pub fn build_opml(subscriptions: &[Subscription]) -> opml::OPML {
    let mut opml = opml::OPML::default();
    for subscription in subscriptions {
        opml.body.outlines.push(opml::Outline {
            text: subscription.podcast.to_string(),
            r#type: Some("rss".to_string()),
            is_comment: None,
            is_breakpoint: None,
            created: Some(subscription.created.to_string()),
            category: None,
            outlines: vec![],
            xml_url: Some(subscription.podcast.to_string()),
            description: None,
            html_url: None,
            language: None,
            title: Some(subscription.podcast.to_string()),
            version: None,
            url: None,
        });
    }
    opml
}
