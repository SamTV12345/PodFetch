use crate::controllers::podcast_episode_controller::TimelineQueryParams;
use crate::dbconfig::DBType;
use crate::models::episode::Episode;
use crate::models::favorites::Favorite;
use crate::models::filter::Filter;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::utils::error::{map_db_error, CustomError};
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::RunQueryDsl;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(PodcastEpisode, Podcast, Option<Episode>, Option<Favorite>)>,
    pub total_elements: i64,
}

impl TimelineItem {
    pub fn get_timeline(
        username_to_search: String,
        conn: &mut DBType,
        favored_only: TimelineQueryParams,
    ) -> Result<TimelineItem, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcasts::id as pid;

        use crate::dbconfig::schema::episodes as phi_struct;
        use crate::dbconfig::schema::episodes::episode as ehid;
        use crate::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::dbconfig::schema::episodes::username as phi_username;
        use crate::dbconfig::schema::favorites::dsl::*;
        use crate::dbconfig::schema::favorites::podcast_id as f_podcast_id;
        use crate::dbconfig::schema::favorites::username as f_username;
        use crate::dbconfig::schema::podcast_episodes::podcast_id as e_podcast_id;

        Filter::save_decision_for_timeline(
            username_to_search.clone(),
            conn,
            favored_only.favored_only,
        );

        let (ph1, ph2) = diesel::alias!(phi_struct as ph1, phi_struct as ph2);

        let subquery = ph2
            .select(max(ph2.field(phistory_date)))
            .filter(ph2.field(ehid).eq(episode_id))
            .filter(ph2.field(phi_username).eq(username_to_search.clone()))
            .group_by(ph2.field(ehid));

        let part_query = podcast_episodes
            .inner_join(podcasts.on(e_podcast_id.eq(pid)))
            .left_join(ph1.on(ph1.field(ehid).eq(episode_id)))
            .filter(
                ph1.field(phistory_date)
                    .nullable()
                    .eq_any(subquery.clone())
                    .or(ph1.field(phistory_date).is_null()),
            )
            .left_join(
                favorites.on(f_username
                    .eq(username_to_search.clone())
                    .and(f_podcast_id.eq(pid))),
            );

        let mut query = part_query
            .clone()
            .order(date_of_recording.desc())
            .limit(20)
            .into_boxed();

        let mut total_count = part_query.clone().count().into_boxed();

        match favored_only.favored_only {
            true => {
                if let Some(last_id) = favored_only.last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id.clone()));
                }

                query = query.filter(f_username.eq(username_to_search.clone()));
                query = query.filter(favored.eq(true));
                total_count = total_count.filter(f_username.eq(username_to_search.clone()));
            }
            false => {
                if let Some(last_id) = favored_only.last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id));
                }
            }
        }

        if favored_only.not_listened {
            query = query.filter(ph1.field(phistory_date).nullable().ne_all(subquery.clone()));
            total_count = total_count.filter(ph1.field(phistory_date).nullable().ne_all(subquery));
        }
        let results = total_count.get_result::<i64>(conn).map_err(map_db_error)?;
        let result = query
            .load::<(PodcastEpisode, Podcast, Option<Episode>, Option<Favorite>)>(conn)
            .map_err(map_db_error)?;

        Ok(TimelineItem {
            total_elements: results,
            data: result,
        })
    }
}
