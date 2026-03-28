use crate::podcast_episode_dto::PodcastEpisodeDto;
use podfetch_persistence::db::get_connection;
use podfetch_persistence::schema::favorite_podcast_episodes::dsl::favorite_podcast_episodes;
use podfetch_persistence::episode::EpisodeEntity as Episode;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_persistence::favorite::FavoriteEntity as Favorite;
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use crate::services::filter::service::FilterService;
use common_infrastructure::error::ErrorSeverity::Critical;
use common_infrastructure::error::{CustomError, map_db_error};
use diesel::RunQueryDsl;
use diesel::dsl::max;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::user::User;
use crate::podcast::PodcastDto;
use crate::podcast::map_podcast_to_dto;
use crate::podcast_episode::{TimelineFavorite, TimelineQueryParams};
use crate::history::EpisodeDto;
use crate::history::map_episode_to_dto;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(
        PodcastEpisodeDto,
        PodcastDto,
        Option<EpisodeDto>,
        Option<TimelineFavorite>,
    )>,
    pub total_elements: i64,
}

#[derive(Queryable, Clone)]
#[diesel(table_name = podfetch_persistence::schema::favorite_podcast_episodes)]
struct JoinedFavoritePodcastEpisode {
    username: String,
    episode_id: i32,
    favorite: bool,
}

impl From<JoinedFavoritePodcastEpisode> for FavoritePodcastEpisode {
    fn from(value: JoinedFavoritePodcastEpisode) -> Self {
        Self {
            username: value.username,
            episode_id: value.episode_id,
            favorite: value.favorite,
        }
    }
}

impl TimelineItem {
    pub fn get_timeline(
        user: User,
        favored_only: TimelineQueryParams,
    ) -> Result<TimelineItem, CustomError> {
        use podfetch_persistence::schema::podcast_episodes::dsl::*;
        use podfetch_persistence::schema::podcasts::dsl::*;
        use podfetch_persistence::schema::podcasts::id as pid;

        use podfetch_persistence::schema::episodes as phi_struct;
        use podfetch_persistence::schema::episodes::episode as ehid;
        use podfetch_persistence::schema::episodes::guid as eguid;
        use podfetch_persistence::schema::episodes::timestamp as phistory_date;
        use podfetch_persistence::schema::episodes::username as phi_username;
        use podfetch_persistence::schema::favorite_podcast_episodes::episode_id as fpe_fav;
        use podfetch_persistence::schema::favorite_podcast_episodes::favorite as idpe_fav_liked;
        use podfetch_persistence::schema::favorite_podcast_episodes::username as idpe_fav;
        use podfetch_persistence::schema::favorites::dsl::*;
        use podfetch_persistence::schema::favorites::podcast_id as f_podcast_id;
        use podfetch_persistence::schema::favorites::username as f_username;
        use podfetch_persistence::schema::podcast_episodes::guid as pguid;
        use podfetch_persistence::schema::podcast_episodes::id as e_p_id;
        use podfetch_persistence::schema::podcast_episodes::podcast_id as e_podcast_id;

        let username_to_search = &user.username;

        let _ = FilterService::default_service()
            .save_timeline_decision(username_to_search, favored_only.favored_only.unwrap_or(false));

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

        match favored_only.favored_only.unwrap_or(false) {
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
            query = query.filter(ph1.field(phistory_date).is_null());
            total_count = total_count.filter(ph1.field(phistory_date).is_null());
        }

        let results = total_count
            .get_result::<i64>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        let result = query
            .load::<(
                PodcastEpisode,
                Podcast,
                Option<JoinedFavoritePodcastEpisode>,
                Option<Episode>,
                Option<Favorite>,
            )>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?
            .into_iter()
            .map(
                |(podcast_episode, podcast, fav_episode, history, favorite)| {
                    let history_dto = history.as_ref().map(|episode| map_episode_to_dto(&episode.clone().into()));
                    (
                        PodcastEpisodeDto::from((
                            podcast_episode,
                            Some(user.clone()),
                            fav_episode.map(Into::into),
                        )),
                        map_podcast_to_dto(podcast.into()),
                        history_dto,
                        favorite.map(|f| TimelineFavorite { favored: f.favored }),
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


