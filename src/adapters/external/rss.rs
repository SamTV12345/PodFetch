use rss::Channel;

#[derive(Clone)]
pub struct PodcastParsed {
    pub title: String,
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
        let language = rss_feed.language.unwrap_or("en".to_string());
        let build_date = rss_feed.last_build_date.unwrap_or_default();
        let keywords = match rss_feed.itunes_ext.clone() {
            Some(itunes_ext) => itunes_ext.keywords.unwrap_or_default(),
            None => String::new(),
        };
        let summary = match rss_feed.itunes_ext.clone() {
            Some(itunes_ext) => itunes_ext.summary.unwrap_or_default(),
            None => String::new(),
        };

        let explicit = match rss_feed.itunes_ext {
            Some(itunes_ext) => itunes_ext.explicit.unwrap_or_default(),
            None => String::new(),
        };

        PodcastParsed {
            title,
            language,
            explicit,
            keywords,
            summary,
            date: build_date,
        }
    }
}
