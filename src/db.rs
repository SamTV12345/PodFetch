use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::dsl::favorite_podcast_episodes;
use crate::controllers::podcast_episode_controller::TimelineQueryParams;
use crate::models::episode::Episode;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::favorites::Favorite;
use crate::models::filter::Filter;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::utils::error::{map_db_error, CustomError};
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::RunQueryDsl;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(
        PodcastEpisodeDto,
        PodcastDto,
        Option<Episode>,
        Option<Favorite>,
    )>,
    pub total_elements: i64,
}

impl TimelineItem {
    pub fn get_timeline(
        user: User,
        favored_only: TimelineQueryParams,
    ) -> Result<TimelineItem, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as pid;

        use crate::adapters::persistence::dbconfig::schema::episodes as phi_struct;
        use crate::adapters::persistence::dbconfig::schema::episodes::episode as ehid;
        use crate::adapters::persistence::dbconfig::schema::episodes::guid as eguid;
        use crate::adapters::persistence::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as phi_username;
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::episode_id as fpe_fav;
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::favorite as idpe_fav_liked;
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::username as idpe_fav;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::favorites::podcast_id as f_podcast_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::username as f_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::guid as pguid;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::id as e_p_id;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::podcast_id as e_podcast_id;

        let username_to_search = &user.username;

        Filter::save_decision_for_timeline(username_to_search, favored_only.favored_only);

        let (ph1, ph2) = diesel::alias!(phi_struct as ph1, phi_struct as ph2);

        let subquery = ph2
            .select(max(ph2.field(phistory_date)))
            .filter(ph2.field(phi_username).eq(&username_to_search))
            .group_by(ph2.field(ehid));

        let part_query = podcast_episodes
            .inner_join(podcasts.on(e_podcast_id.eq(pid)))
            .left_join(
                favorite_podcast_episodes
                    .on(e_p_id.eq(fpe_fav).and(idpe_fav.eq(&username_to_search))),
            )
            .left_join(ph1.on(ph1.field(eguid).eq(pguid.nullable())))
            .filter(
                ph1.field(phistory_date)
                    .nullable()
                    .eq_any(subquery)
                    .or(ph1.field(phistory_date).is_null()),
            )
            .left_join(favorites.on(f_username.eq(&username_to_search).and(f_podcast_id.eq(pid))));

        let mut query = part_query
            .order(date_of_recording.desc())
            .limit(20)
            .into_boxed();

        let mut total_count = part_query.count().into_boxed();

        match favored_only.favored_only {
            true => {
                if let Some(last_id) = favored_only.last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id.clone()));
                }

                query = query.filter(f_username.eq(&username_to_search));
                query = query.filter(favored.eq(true));
                total_count = total_count.filter(f_username.eq(&username_to_search));
            }
            false => {
                if let Some(last_id) = favored_only.last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id));
                }
            }
        }

        if favored_only.favored_episodes {
            query = query.filter(idpe_fav_liked.eq(true));
            total_count = total_count.filter(idpe_fav_liked.eq(true));
        }

        if favored_only.not_listened {
            query = query.filter(ph1.field(phistory_date).nullable().ne_all(subquery));
            total_count = total_count.filter(ph1.field(phistory_date).nullable().ne_all(subquery));
        }
        let results = total_count
            .get_result::<i64>(&mut get_connection())
            .map_err(map_db_error)?;
        let result = query
            .load::<(
                PodcastEpisode,
                Podcast,
                Option<FavoritePodcastEpisode>,
                Option<Episode>,
                Option<Favorite>,
            )>(&mut get_connection())
            .map_err(map_db_error)?
            .into_iter()
            .map(
                |(podcast_episode, podcast, fav_episode, history, favorite)| {
                    (
                        PodcastEpisodeDto::from((podcast_episode, Some(user.clone()), fav_episode)),
                        PodcastDto::from(podcast),
                        history,
                        favorite,
                    )
                },
            )
            .collect();

        Ok(TimelineItem {
            total_elements: results,
            data: result,
        })
    }
}
