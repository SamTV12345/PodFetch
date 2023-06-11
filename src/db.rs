
use crate::constants::constants::{DEFAULT_SETTINGS, STANDARD_USER};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_dto::PodcastDto;

use crate::models::podcasts::Podcast;
use crate::models::settings::Setting;
use crate::service::mapping_service::MappingService;
use crate::utils::podcast_builder::PodcastExtra;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::dsl::{sql};
use diesel::prelude::*;
use diesel::{insert_into, sql_query, RunQueryDsl, delete, debug_query};
use rss::Item;
use std::io::Error;

use std::sync::MutexGuard;

use diesel::query_builder::{QueryBuilder};
use diesel::sql_types::{Text, Timestamp};
use crate::controllers::podcast_episode_controller::TimelineQueryParams;

use crate::{DbConnection, MyQueryBuilder};
use crate::dbconfig::__sqlite_schema::podcast_history_items::dsl::podcast_history_items;
use crate::models::episode::{Episode, EpisodeAction};
use crate::models::favorites::Favorite;
use crate::models::filter::Filter;
use crate::models::order_criteria::{OrderCriteria, OrderOption};

use crate::utils::do_retry::do_retry;
use crate::utils::time::opt_or_empty_string;


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem{
    pub data: Vec<(PodcastEpisode,Podcast, Option<Favorite>)>,
    pub total_elements: i64
}

pub struct DB {
    mapping_service: MappingService,
}

impl Clone for DB {
    fn clone(&self) -> Self {
        DB {
            mapping_service: MappingService::new(),
        }
    }
}

impl DB {
    pub fn new() -> Result<DB, String> {
        Ok(DB {
            mapping_service: MappingService::new(),
        })
    }








    pub fn get_timeline(username_to_search: String, conn: &mut DbConnection, favored_only: TimelineQueryParams)
        -> TimelineItem {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcasts::id as pid;
        
        use crate::dbconfig::schema::favorites::dsl::*;
        use crate::dbconfig::schema::favorites::username as f_username;
        use crate::dbconfig::schema::favorites::podcast_id as f_podcast_id;
        use crate::dbconfig::schema::podcast_episodes::podcast_id as e_podcast_id;

        Filter::save_decision_for_timeline(username_to_search.clone(),conn,favored_only.favored_only);

        let mut query = podcast_episodes.inner_join(podcasts.on(e_podcast_id.eq(pid)))
            .left_join(favorites.on(f_username.eq(username_to_search.clone()).and(f_podcast_id.eq(pid))))
            .order(date_of_recording.desc())
            .limit(20)
            .into_boxed();

        let mut total_count = podcast_episodes.inner_join(podcasts.on(e_podcast_id.eq(pid)))
            .left_join(favorites.on(f_username.eq(username_to_search.clone()).and(f_podcast_id.eq(pid))))
            .order(date_of_recording.desc())
            .count()
            .into_boxed();

        match favored_only.favored_only {
            true=>{
                match favored_only.last_timestamp {
                    Some(last_id) => {
                        query = query.filter(date_of_recording.lt(last_id.clone()));
                    }
                    None => {}
                }

                query = query.filter(f_username.eq(username_to_search.clone()));
                total_count = total_count.filter(f_username.eq(username_to_search.clone()));

            }
            false=>{
                match favored_only.last_timestamp {
                    Some(last_id) => {
                        query = query.filter(date_of_recording.lt(last_id));
                    }
                    None => {}
                }
            }
        }
        let results = total_count.get_result::<i64>(conn).expect("Error counting results");
        let result = query.load::<(PodcastEpisode, Podcast, Option<Favorite>)>(conn).expect("Error \
        loading podcast episode by id");

        TimelineItem{
            total_elements: results,
            data: result
        }

    }



    pub fn get_podcast_by_rss_feed(rss_feed_1:String, conn: &mut DbConnection) -> Podcast {
        use crate::dbconfig::schema::podcasts::dsl::*;

        podcasts
            .filter(rssfeed.eq(rss_feed_1))
            .first::<Podcast>(conn)
            .expect("Error loading podcast by rss feed")
    }



}
