use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct Subscription {
    pub id: i32,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub created: chrono::NaiveDateTime,
    pub deleted: Option<chrono::NaiveDateTime>,
}

impl From<podfetch_domain::subscription::Subscription> for Subscription {
    fn from(value: podfetch_domain::subscription::Subscription) -> Self {
        Self {
            id: value.id,
            username: value.username,
            device: value.device,
            podcast: value.podcast,
            created: value.created,
            deleted: value.deleted,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct SubscriptionChangesToClient {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}

impl From<podfetch_domain::subscription::SubscriptionChangesToClient>
    for SubscriptionChangesToClient
{
    fn from(value: podfetch_domain::subscription::SubscriptionChangesToClient) -> Self {
        Self {
            add: value.add,
            remove: value.remove,
            timestamp: value.timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionModelChanges {
    pub add: Vec<Subscription>,
    pub remove: Vec<Subscription>,
    pub timestamp: i64,
}

impl From<podfetch_domain::subscription::SubscriptionModelChanges> for SubscriptionModelChanges {
    fn from(value: podfetch_domain::subscription::SubscriptionModelChanges) -> Self {
        Self {
            add: value.add.into_iter().map(Into::into).collect(),
            remove: value.remove.into_iter().map(Into::into).collect(),
            timestamp: value.timestamp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderAvailablePodcast {
    pub device: String,
    pub podcast: String,
}

impl From<podfetch_domain::subscription::GPodderAvailablePodcast> for GPodderAvailablePodcast {
    fn from(value: podfetch_domain::subscription::GPodderAvailablePodcast) -> Self {
        Self {
            device: value.device,
            podcast: value.podcast,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct SubscriptionRetrieveRequest {
    #[serde(default)]
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
    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error>;
}

pub fn to_client_changes(changes: SubscriptionModelChanges) -> SubscriptionChangesToClient {
    SubscriptionChangesToClient {
        add: changes
            .add
            .iter()
            .map(|subscription| subscription.podcast.clone())
            .collect(),
        remove: changes
            .remove
            .iter()
            .map(|subscription| subscription.podcast.clone())
            .collect(),
        timestamp: changes.timestamp,
    }
}

pub fn build_opml(subscriptions: &[Subscription]) -> opml::OPML {
    let mut opml = opml::OPML::default();
    opml.head = Some(opml::Head {
        title: Some("PodFetch Subscriptions".to_string()),
        ..opml::Head::default()
    });

    // Kodi's OPML importer expects body/head maps, not self-closing XML nodes.
    // Keep one non-RSS placeholder outline for empty subscription exports.
    if subscriptions.is_empty() {
        opml.body.outlines.push(opml::Outline {
            text: "PodFetch Subscriptions".to_string(),
            title: Some("PodFetch Subscriptions".to_string()),
            ..opml::Outline::default()
        });
        return opml;
    }

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
