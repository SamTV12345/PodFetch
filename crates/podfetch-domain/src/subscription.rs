use chrono::{NaiveDateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    pub id: i32,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub created: NaiveDateTime,
    pub deleted: Option<NaiveDateTime>,
}

impl Subscription {
    pub fn new(username: String, device: String, podcast: String) -> Self {
        Self {
            id: 0,
            username,
            device,
            podcast,
            created: Utc::now().naive_utc(),
            deleted: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionChangesToClient {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionModelChanges {
    pub add: Vec<Subscription>,
    pub remove: Vec<Subscription>,
    pub timestamp: i64,
}

impl From<SubscriptionModelChanges> for SubscriptionChangesToClient {
    fn from(value: SubscriptionModelChanges) -> Self {
        Self {
            add: value
                .add
                .iter()
                .map(|subscription| subscription.podcast.clone())
                .collect(),
            remove: value
                .remove
                .iter()
                .map(|subscription| subscription.podcast.clone())
                .collect(),
            timestamp: value.timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GPodderAvailablePodcast {
    pub device: String,
    pub podcast: String,
}

pub trait SubscriptionRepository: Send + Sync {
    type Error;

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;
    fn get_device_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error>;
    fn get_user_subscriptions(
        &self,
        username: &str,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error>;
    fn update_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<Vec<String>>, Self::Error>;
    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error>;
    fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        username: &str,
    ) -> Result<Vec<String>, Self::Error>;
}
