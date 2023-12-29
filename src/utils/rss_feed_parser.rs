use rss::Channel;

#[derive(Clone)]
pub struct PodcastParsed {
    pub title: String,
    pub description: String,
    pub language: String,
    pub explicit: String,
    pub keywords: String,
    pub summary: String,
    pub date: String,
}

pub struct RSSFeedParser;

impl RSSFeedParser {
    pub fn parse_rss_feed(rss_feed: Channel) -> PodcastParsed {
        let title = rss_feed.title;
        let description = rss_feed.description;
        let language = rss_feed.language.unwrap_or("en".to_string());
        let build_date = rss_feed.last_build_date.unwrap_or("".to_string());
        let keywords = match rss_feed.itunes_ext.clone() {
            Some(itunes_ext) => itunes_ext.keywords.unwrap_or("".to_string()),
            None => "".to_string(),
        };
        let summary = match rss_feed.itunes_ext.clone() {
            Some(itunes_ext) => itunes_ext.summary.unwrap_or("".to_string()),
            None => "".to_string(),
        };

        let explicit = match rss_feed.itunes_ext {
            Some(itunes_ext) => itunes_ext.explicit.unwrap_or("".to_string()),
            None => "".to_string(),
        };

        PodcastParsed {
            title,
            description,
            language,
            explicit,
            keywords,
            summary,
            date: build_date,
        }
    }
}
