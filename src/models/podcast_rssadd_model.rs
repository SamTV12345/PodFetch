use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastRSSAddModel {
    #[serde(rename = "rssFeedUrl")]
    pub rss_feed_url: String,
}
