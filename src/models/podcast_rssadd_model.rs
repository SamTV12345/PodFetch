#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct PodcastRSSAddModel{
    #[serde(rename = "rssFeedUrl")]
    pub rss_feed_url: String,
}