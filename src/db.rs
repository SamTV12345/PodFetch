use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use diesel::prelude::*;
use diesel::{RunQueryDsl};
use crate::controllers::podcast_episode_controller::TimelineQueryParams;
use crate::{DbConnection};
use crate::models::favorites::Favorite;
use crate::models::filter::Filter;
use crate::utils::error::{CustomError, map_db_error};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(PodcastEpisode, Podcast, Option<Favorite>)>,
    pub total_elements: i64,
}

impl TimelineItem {
    pub fn get_timeline(username_to_search: String, conn: &mut DbConnection, favored_only: TimelineQueryParams)
                        -> Result<TimelineItem, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcasts::id as pid;

        use crate::dbconfig::schema::favorites::dsl::*;
        use crate::dbconfig::schema::favorites::username as f_username;
        use crate::dbconfig::schema::favorites::podcast_id as f_podcast_id;
        use crate::dbconfig::schema::podcast_episodes::podcast_id as e_podcast_id;

        Filter::save_decision_for_timeline(username_to_search.clone(), conn, favored_only.favored_only);

        let mut query = podcast_episodes.inner_join(podcasts.on(e_podcast_id.eq(pid)))
            .left_join(favorites.on(f_username.eq(username_to_search.clone()).and(f_podcast_id.eq(pid))))
            .order(date_of_recording.desc())
            .limit(20)
            .into_boxed();

        let mut total_count = podcast_episodes.inner_join(podcasts.on(e_podcast_id.eq(pid)))
            .left_join(favorites.on(f_username.eq(username_to_search.clone()).and(f_podcast_id.eq(pid))))
            .count()
            .into_boxed();


        match favored_only.favored_only {
            true => {
                if let Some(last_id) = favored_only.last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id.clone()));
                }


                query = query.filter(f_username.eq(username_to_search.clone()));
                total_count = total_count.filter(f_username.eq(username_to_search.clone()));
            }
            false => {
                           if let Some(last_id) = favored_only.last_timestamp {
                                 query = query.filter(date_of_recording.lt(last_id));
                  }

            }
        }
        let results = total_count.get_result::<i64>(conn).expect("Error counting results");
        let result = query.load::<(PodcastEpisode, Podcast, Option<Favorite>)>(conn).map_err
        (map_db_error)?;

        Ok(TimelineItem {
            total_elements: results,
            data: result,
        })
    }
}
