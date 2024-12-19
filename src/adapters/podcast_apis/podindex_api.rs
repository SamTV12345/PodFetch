#[derive(Deserialize)]
pub struct ByFeedPodindex {
    status: String,
    query: FeedQuery,
    feed: PodindexFeed
}

#[derive(Deserialize)]
pub struct PodindexFeed {
    id: i32,
    podcast_guid: String,
    title: String,
    url: String,
    original_url: String,
    link: String,
    description: String,
    author: String,
    owner_name: String,
    image: String,
    artwork: String,
    last_update_time: i32,
    last_crawl_time: i32,
    last_parse_time: i32,
    last_good_http_status_time: i32,
    last_http_status: i32,
    content_type: String,
    itunes_id: Option<i32>,
    itunes_type: String,
    generator: String,
    language: String,
    explicit: bool,
    r#type: i32,
    medium: String,
    dead: i32,
    chash: String,
    episodes_count: i32,
    crawl_errors: i32,
    parse_errors: i32,
    locked: i32,
    image_url_hash: i32,
}

#[derive(Deserialize)]
pub struct FeedQuery {
    id: String
}

#[cfg(test)]
mod tests {
    use crate::adapters::podcast_apis::podindex_api::ByFeedPodindex;

    #[test]
    pub fn test_podindex_api_by_feed_model() {
        let feed_string = r#"
        {
  "status": "true",
  "query": {
    "id": "920666"
  },
  "feed": {
    "id": 75075,
    "podcastGuid": "9b024349-ccf0-5f69-a609-6b82873eab3c",
    "title": "Batman University",
    "url": "https://feeds.theincomparable.com/batmanuniversity",
    "originalUrl": "https://feeds.theincomparable.com/batmanuniversity",
    "link": "https://www.theincomparable.com/batmanuniversity/",
    "description": "Batman University is a seasonal podcast about you know who. It began with an analysis of episodes of “Batman: The Animated Series” but has now expanded to cover other series, movies, and media. Your professor is Tony Sindelar.",
    "author": "Tony Sindelar",
    "ownerName": "The Incomparable",
    "image": "https://www.theincomparable.com/imgs/logos/logo-batmanuniversity-3x.jpg?cache-buster=2019-06-11",
    "artwork": "https://www.theincomparable.com/imgs/logos/logo-batmanuniversity-3x.jpg?cache-buster=2019-06-11",
    "lastUpdateTime": 1613394044,
    "lastCrawlTime": 1613394034,
    "lastParseTime": 1613394045,
    "lastGoodHttpStatusTime": 1613394034,
    "lastHttpStatus": 200,
    "contentType": "application/rss+xml",
    "itunesId": 1441923632,
    "itunesType": "episodic",
    "generator": "my podcast host",
    "language": "en-us",
    "explicit": false,
    "type": 0,
    "medium": "music",
    "dead": 0,
    "chash": "ad651c60eaaf3344595c0dd0bd787993",
    "episodeCount": 19,
    "crawlErrors": 0,
    "parseErrors": 0,
    "categories": {
      "104": "Tv",
      "105": "Film",
      "107": "Reviews"
    },
    "locked": 0,
    "imageUrlHash": 3969216649,
    "value": {
      "model": {
        "type": "lightning",
        "method": "keysend",
        "suggested": "0.00000020000"
      },
      "destinations": [
        {
          "name": "podcaster",
          "address": "03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a",
          "type": "node",
          "split": 99,
          "fee": true,
          "customKey": "112111100",
          "customValue": "wal_ZmqFg13NB31oek"
        }
      ]
    },
    "funding": {
      "url": "https://patreon.com/johnchidgey",
      "message": "Pragmatic on Patreon"
    }
  },
  "description": "Found matching feed"
}"#;

        let result = serde_json::from_str::<ByFeedPodindex>(feed_string).unwrap();
        assert_eq!(result.status, "true");
        assert_eq!(result.feed.title, "Batman University");
        assert_eq!(result.feed.url, "https://feeds.theincomparable.com/batmanuniversity");
        assert_eq!(result.feed.image, "https://www.theincomparable.com/imgs/logos/logo-batmanuniversity-3x.jpg?cache-buster=2019-06-11");
    }
}