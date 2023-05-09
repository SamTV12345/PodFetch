use crate::config::dbconfig::establish_connection;
use crate::constants::constants::{DEFAULT_SETTINGS, STANDARD_USER};
use crate::models::itunes_models::{Podcast, PodcastDto, PodcastEpisode};
use crate::models::models::{
    Notification, PodcastHistoryItem, PodcastWatchedEpisodeModelWithPodcastEpisode,
    PodcastWatchedPostModel,
};
use crate::models::settings::Setting;
use crate::service::mapping_service::MappingService;
use crate::utils::podcast_builder::PodcastExtra;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::{insert_into, sql_query, RunQueryDsl, delete};
use rss::Item;
use std::io::Error;
use std::sync::MutexGuard;
use std::time::SystemTime;
use diesel::sql_types::{Text, Timestamp};
use crate::models::episode::{Episode, EpisodeAction};
use crate::models::favorites::Favorite;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::utils::do_retry::do_retry;
use crate::utils::time::opt_or_empty_string;

pub struct DB {
    conn: SqliteConnection,
    mapping_service: MappingService,
}

impl Clone for DB {
    fn clone(&self) -> Self {
        DB {
            conn: establish_connection(),
            mapping_service: MappingService::new(),
        }
    }
}
impl DB {
    pub fn new() -> Result<DB, String> {
        let conn = establish_connection();
        Ok(DB {
            conn,
            mapping_service: MappingService::new(),
        })
    }

    pub fn find_by_rss_feed_url(conn:&mut SqliteConnection, feed_url: &str) -> Option<Podcast> {
        use crate::schema::podcasts::dsl::*;
        podcasts
            .filter(rssfeed.eq(feed_url))
            .first::<Podcast>(conn)
            .optional()
            .expect("Error loading podcast by rss feed url")
    }

    pub fn get_podcasts(conn: &mut SqliteConnection, u: String, mapping_service: MutexGuard<MappingService>)
        -> Result<Vec<PodcastDto>, String> {
        use crate::schema::podcasts::dsl::podcasts;
        use crate::schema::favorites::dsl::favorites as f_db;
        use crate::schema::favorites::dsl::podcast_id as f_id;
        use crate::schema::podcasts::id as p_id;
        use crate::schema::favorites::dsl::username;
        let result = podcasts
            .left_join(f_db.on(username.eq(u).and(f_id.eq(p_id))))
            .load::<(Podcast, Option<Favorite>)>(conn)
            .expect("Error loading podcasts");

        let mapped_result = result
            .iter()
            .map(|podcast| return mapping_service.map_podcast_to_podcast_dto_with_favorites
            (&*podcast))
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }


    pub fn get_all_podcasts(conn: &mut SqliteConnection)
        -> Result<Vec<Podcast>, String> {
        use crate::schema::podcasts::dsl::podcasts;
        let result = podcasts
            .load::<Podcast>(conn)
            .expect("Error loading podcasts");
        Ok(result)
    }

    pub fn get_podcast(conn: &mut SqliteConnection, podcast_id_to_be_found: i32) -> Result<Podcast, Error> {
        use crate::schema::podcasts::dsl::podcasts;
        use crate::schema::podcasts::id as podcast_id;
        let found_podcast = podcasts
            .filter(podcast_id.eq(podcast_id_to_be_found))
            .first::<Podcast>(conn)
            .optional()
            .expect("Error loading podcast by id");

        match found_podcast {
            Some(podcast) => Ok(podcast),
            None => Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Podcast not found",
            )),
        }
    }

    pub fn delete_podcast(conn: &mut SqliteConnection, podcast_id_to_find: i32){
        use crate::schema::podcasts::dsl::*;
        delete(podcasts.filter(id.eq(podcast_id_to_find)))
            .execute(conn)
            .expect("Error deleting podcast");
    }

    pub fn get_podcast_episode_by_id(
        conn: &mut SqliteConnection,
        podcas_episode_id_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(episode_id.eq(podcas_episode_id_to_be_found))
            .first::<PodcastEpisode>(conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode)
    }

    pub fn get_podcast_episode_by_url(
        conn: &mut SqliteConnection,
        podcas_episode_url_to_be_found: &str,
        i: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::dsl::*;
        let found_podcast_episode;
        if i.is_some(){
            found_podcast_episode = podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found).and(podcast_id.eq(i.unwrap())))
                .first::<PodcastEpisode>(conn)
                .optional()
                .expect("Error loading podcast by id");
        }
        else{
            found_podcast_episode = podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found))
                .first::<PodcastEpisode>(conn)
                .optional()
                .expect("Error loading podcast by id");
        }


        Ok(found_podcast_episode)
    }

    pub fn query_podcast_episode_by_url(
        conn: &mut SqliteConnection,
        podcas_episode_url_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(url.like("%".to_owned()+podcas_episode_url_to_be_found+"%"))
            .first::<PodcastEpisode>(conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode)
    }

    pub fn get_podcast_by_track_id(conn: &mut SqliteConnection, podcast_id: i32) -> Result<Option<Podcast>, String> {
        use crate::schema::podcasts::directory_id;
        use crate::schema::podcasts::dsl::podcasts;
        let optional_podcast = podcasts
            .filter(directory_id.eq(podcast_id.to_string()))
            .first::<Podcast>(conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(optional_podcast)
    }

    pub fn insert_podcast_episodes(
        conn: &mut SqliteConnection,
        podcast: Podcast,
        item: Item,
        optional_image: Option<String>,
        duration: i32,
    ) -> PodcastEpisode {
        use crate::schema::podcast_episodes::dsl::*;
        let uuid_podcast = uuid::Uuid::new_v4();

        let mut inserted_date = "".to_string();
        let inserted_image_url;
        match &item.pub_date {
            Some(date) => {
                let date = DateTime::parse_from_rfc2822(date).expect("Error parsing date");
                inserted_date = date.to_rfc3339()
            }
            None => {}
        }

        match optional_image {
            Some(image_url_podcast_episode) => {
                inserted_image_url = image_url_podcast_episode;
            }
            None => {
                inserted_image_url = podcast.original_image_url;
            }
        }

        let inserted_podcast = insert_into(podcast_episodes)
            .values((
                total_time.eq(duration),
                podcast_id.eq(podcast.id),
                episode_id.eq(uuid_podcast.to_string()),
                name.eq(item.title.as_ref().unwrap().clone()),
                url.eq(item.enclosure.unwrap().url),
                date_of_recording.eq(inserted_date),
                image_url.eq(inserted_image_url),
                description.eq(opt_or_empty_string(item.description)),
            ))
            .get_result::<PodcastEpisode>(conn)
            .expect("Error inserting podcast episode");
        return inserted_podcast;
    }

    pub fn add_podcast_to_database(
        conn: &mut SqliteConnection,
        collection_name: String,
        collection_id: String,
        feed_url: String,
        image_url_1: String,
        directory_name_to_insert: String
    ) -> Podcast {
        use crate::schema::podcasts;
        use crate::schema::podcasts::{directory_id, image_url, name as podcast_name, rssfeed};
        use crate::schema::podcasts::{original_image_url, directory_name};

        let inserted_podcast = insert_into(podcasts::table)
            .values((
                directory_id.eq(collection_id.to_string()),
                podcast_name.eq(collection_name.to_string()),
                rssfeed.eq(feed_url.to_string()),
                image_url.eq(image_url_1.to_string()),
                original_image_url.eq(image_url_1.to_string()),
                directory_name.eq(directory_name_to_insert.to_string())
            ))
            .get_result::<Podcast>(conn)
            .expect("Error inserting podcast");
        return inserted_podcast;
    }

    pub fn get_last_n_podcast_episodes(
        conn: &mut SqliteConnection,
        podcast_episode_id: i32,
        number_to_download: i32
    ) -> Result<Vec<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::dsl::*;
        let podcasts = podcast_episodes
            .filter(podcast_id.eq(podcast_episode_id))
            .limit(number_to_download as i64)
            .order(date_of_recording.desc())
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcasts");
        Ok(podcasts)
    }

    pub fn get_podcast_episodes_of_podcast(
        conn: &mut SqliteConnection,
        podcast_id_to_be_searched: i32,
        last_id: Option<String>,
    ) -> Result<Vec<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::*;
        use crate::schema::podcast_episodes::dsl::podcast_episodes;
        match last_id {
            Some(last_id) => {
                let podcasts_found = podcast_episodes
                    .filter(podcast_id.eq(podcast_id_to_be_searched))
                    .filter(date_of_recording.lt(last_id))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(conn)
                    .expect("Error loading podcasts");
                Ok(podcasts_found)
            }
            None => {
                let podcasts_found = podcast_episodes
                    .filter(podcast_id.eq(podcast_id_to_be_searched))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(conn)
                    .expect("Error loading podcasts");

                Ok(podcasts_found)
            }
        }
    }

    pub fn log_watchtime(conn: &mut SqliteConnection, watch_model: PodcastWatchedPostModel, designated_username: String) -> Result<(), String> {
        let result = Self::
            get_podcast_episode_by_id(conn, &watch_model.podcast_episode_id)
            .unwrap();

        use crate::schema::podcast_history_items::dsl::*;
        match result {
            Some(result) => {
                let now = SystemTime::now();
                let now: DateTime<Utc> = now.into();
                let now: &str = &now.to_rfc3339();
                insert_into(podcast_history_items)
                    .values((
                        podcast_id.eq(result.podcast_id),
                        episode_id.eq(result.episode_id),
                        watched_time.eq(watch_model.time),
                        date.eq(&now),
                        username.eq(designated_username),
                    ))
                    .execute(conn)
                    .expect("Error inserting podcast episode");
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn delete_watchtime(conn: &mut SqliteConnection, podcast_id_to_delete: i32) -> Result<(),
        String> {
        use crate::schema::podcast_history_items::dsl::*;

                delete(podcast_history_items)
                    .filter(podcast_id.eq(podcast_id_to_delete))
                    .execute(conn)
                    .expect("Error inserting podcast episode");
                Ok(())
    }

    pub fn get_watchtime(
        conn: &mut SqliteConnection,
        podcast_id_tos_search: &str,
        username_to_find: String
    ) -> Result<PodcastHistoryItem, String> {
        let result = Self::get_podcast_episode_by_id(conn, podcast_id_tos_search)
            .unwrap();
        use crate::schema::podcast_history_items::dsl::*;

        match result {
            Some(found_podcast) => {
                let history_item = podcast_history_items
                    .filter(episode_id.eq(podcast_id_tos_search).and(username.eq(username_to_find.clone())))
                    .order(date.desc())
                    .first::<PodcastHistoryItem>(conn)
                    .optional()
                    .expect("Error loading podcast episode by id");

                return match history_item {
                    Some(found_history_item) => {
                         let option_episode = Episode::get_watch_log_by_username_and_episode
                             (username_to_find.clone(), conn, found_podcast.clone().url);
                        if option_episode.is_some(){
                            let episode = option_episode.unwrap();
                            if episode.action == EpisodeAction::Play.to_string() && episode
                                .position.unwrap()>found_history_item.watched_time && episode.timestamp>found_history_item.date{

                                let found_podcast_item = Self::get_podcast(conn, found_history_item
                                    .podcast_id).unwrap();
                                return Ok(Episode::convert_to_podcast_history_item(&episode,
                                                                             found_podcast_item,
                                                                                   found_podcast));
                            }
                        }
                        Ok(found_history_item)
                    },
                    None => Ok(PodcastHistoryItem {
                        id: 0,
                        podcast_id: found_podcast.podcast_id,
                        episode_id: found_podcast.episode_id,
                        watched_time: 0,
                        username: STANDARD_USER.to_string(),
                        date: Utc::now().naive_utc()
                    }),
                };
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn get_last_watched_podcasts(
        &mut self,
        conn: &mut SqliteConnection,
        designated_username: String) -> Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>, String> {
        let result = sql_query(
            "SELECT * FROM (SELECT * FROM podcast_history_items WHERE username=? ORDER BY date \
            DESC) GROUP BY episode_id  LIMIT 10;",
        )
            .bind::<Text,_>(designated_username)
        .load::<PodcastHistoryItem>(&mut self.conn)
        .unwrap();

        let podcast_watch_episode = result
            .iter()
            .map(|podcast_watch_model| {
                let optional_podcast = DB::get_podcast_episode_by_id(conn,&podcast_watch_model
                    .episode_id)
                    .unwrap();
                match optional_podcast {
                    Some(podcast_episode) => {
                        let podcast_dto = self
                            .mapping_service
                            .map_podcastepisode_to_dto(&podcast_episode);
                        let podcast = DB::get_podcast(conn, podcast_episode.podcast_id).unwrap();
                        let podcast_watch_model = self
                            .mapping_service
                            .map_podcast_history_item_to_with_podcast_episode(
                                &podcast_watch_model.clone(),
                                podcast_dto,
                                podcast,
                            );
                        return podcast_watch_model;
                    }
                    None => {
                        panic!("Podcast episode not found");
                    }
                }
            })
            .collect::<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>();
        Ok(podcast_watch_episode)
    }

    pub fn update_total_podcast_time_and_image(
        &mut self,
        episode_id: &str,
        image_url: &str,
        local_download_url: &str,
    ) -> Result<(), String> {
        use crate::schema::podcast_episodes::dsl::episode_id as episode_id_column;
        use crate::schema::podcast_episodes::dsl::local_image_url as local_image_url_column;
        use crate::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::schema::podcast_episodes::dsl::podcast_episodes;
        let result = podcast_episodes
            .filter(episode_id_column.eq(episode_id))
            .first::<PodcastEpisode>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");

        match result {
            Some(..) => {
                diesel::update(podcast_episodes)
                    .filter(episode_id_column.eq(episode_id))
                    .set((
                        local_image_url_column.eq(image_url),
                        local_url_column.eq(local_download_url),
                    ))
                    .execute(&mut self.conn)
                    .expect("Error updating local image url");
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }


    pub fn delete_episodes_of_podcast(conn: &mut SqliteConnection, podcast_id: i32) -> Result<(), String> {
        use crate::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;
        use crate::schema::podcast_episodes::dsl::podcast_episodes;


                delete(podcast_episodes)
                    .filter(podcast_id_column.eq(podcast_id))
                    .execute(conn)
                    .expect("Error deleting podcast episodes");
                Ok(())
    }

    pub fn update_podcast_image(mut self, id: &str, image_url: &str) -> Result<(), String> {
        use crate::schema::podcasts::dsl::directory_id;
        use crate::schema::podcasts::dsl::image_url as image_url_column;
        use crate::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result = dsl_podcast
            .filter(directory_id.eq(id))
            .first::<Podcast>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(..) => {
                diesel::update(dsl_podcast.filter(directory_id.eq(id)))
                    .set(image_url_column.eq(image_url))
                    .execute(&mut self.conn)
                    .expect("Error updating podcast episode");
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn get_podcast_by_directory_id(mut self, podcast_id: &str) -> Result<Option<Podcast>, String> {
        use crate::schema::podcasts::dsl::directory_id;
        use crate::schema::podcasts::dsl::podcasts as dsl_podcast;
        let result = dsl_podcast
            .filter(directory_id.eq(podcast_id))
            .first::<Podcast>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");
        Ok(result)
    }

    pub fn check_if_downloaded(&mut self, download_episode_url: &str) -> Result<bool, String> {
        use crate::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        use crate::schema::podcast_episodes::url as podcast_episode_url;
        let result = dsl_podcast_episodes
            .filter(local_url_column.is_not_null())
            .filter(podcast_episode_url.eq(download_episode_url))
            .first::<PodcastEpisode>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(podcast_episode) => {
                return match podcast_episode.status.as_str() {
                    "N" => Ok(false),
                    "D" => Ok(true),
                    "P" => Ok(false),
                    _ => Ok(false),
                }
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn update_podcast_episode_status(
        &mut self,
        download_url_of_episode: &str,
        status_to_insert: &str,
    ) -> Result<PodcastEpisode, String> {
        use crate::schema::podcast_episodes::dsl::*;

        let updated_podcast =
            diesel::update(podcast_episodes.filter(url.eq(download_url_of_episode)))
                .set((
                    status.eq(status_to_insert),
                    download_time.eq(Utc::now().naive_utc()),
                ))
                .get_result::<PodcastEpisode>(&mut self.conn)
                .expect("Error updating podcast episode");

        Ok(updated_podcast)
    }

    pub fn get_unread_notifications(&mut self) -> Result<Vec<Notification>, String> {
        use crate::schema::notifications::dsl::*;
        let result = notifications
            .filter(status.eq("unread"))
            .order(created_at.desc())
            .load::<Notification>(&mut self.conn)
            .unwrap();
        Ok(result)
    }

    pub fn insert_notification(&mut self, notification: Notification) -> Result<(), String> {
        use crate::schema::notifications::dsl::notifications;
        use crate::schema::notifications::*;
        do_retry(||{insert_into(notifications)
            .values((
                type_of_message.eq(notification.clone().type_of_message),
                message.eq(notification.clone().message),
                status.eq(notification.clone().status),
                created_at.eq(notification.clone().created_at),
            ))
            .execute(&mut self.conn)})
            .expect("Error inserting Notification");
        Ok(())
    }

    pub fn update_status_of_notification(
        &mut self,
        id_to_search: i32,
        status_update: &str,
    ) -> Result<(), String> {
        use crate::schema::notifications::dsl::*;
        do_retry(|| {
            diesel::update(notifications.filter(id.eq(id_to_search)))
                .set(status.eq(status_update))
                .execute(&mut self.conn)
        }).expect("Error updating notification");
        Ok(())
    }

    pub fn query_for_podcast(&mut self, query: &str) -> Result<Vec<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::dsl::*;
        let result = podcast_episodes
            .filter(
                name.like(format!("%{}%", query))
                    .or(description.like(format!("%{}%", query))),
            )
            .load::<PodcastEpisode>(&mut self.conn)
            .expect("Error loading podcast episode by id");
        Ok(result)
    }

    pub fn update_podcast_favor(&mut self, podcast_id_1: &i32, favor: bool, username_1: String) ->
                                                                                               Result<(), String> {
        use crate::schema::favorites::dsl::favored as favor_column;
        use crate::schema::favorites::dsl::favorites as f_db;
        use crate::schema::favorites::dsl::podcast_id;
        use crate::schema::favorites::dsl::username;

        let res = f_db.filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1.clone())))
            .first::<Favorite>(&mut self.conn)
            .optional().unwrap();

        match res{
            Some(..) => {
                diesel::update(f_db.filter(podcast_id.eq(podcast_id_1).and(username.eq(username_1))))
                    .set(favor_column.eq(favor))
                    .execute(&mut self.conn).expect("Error updating podcast");
                Ok(())
            }
            None => {
                insert_into(f_db)
                    .values((
                        podcast_id.eq(podcast_id_1),
                        username.eq(username_1),
                        favor_column.eq(favor),
                    ))
                    .execute(&mut self.conn).map_err(|e| e.to_string()).unwrap();
                Ok(())
            }
        }
    }

    pub fn get_favored_podcasts(&mut self, found_username: String) -> Result<Vec<PodcastDto>,
        String> {
        use crate::schema::podcasts::dsl::podcasts as dsl_podcast;
        use crate::schema::favorites::dsl::favorites as f_db;
        use crate::schema::favorites::dsl::username as user_favor;
        use crate::schema::favorites::dsl::favored as favor_column;
        let result:Vec<(Podcast, Favorite)>;

         result = dsl_podcast
                    .inner_join(f_db)
                    .filter(
                        favor_column.eq(true).and(
                        user_favor.eq(found_username)))
                    .load::<(Podcast, Favorite)>(&mut self.conn).unwrap();



        let mapped_result = result
            .iter()
            .map(|podcast| return self.mapping_service.map_podcast_to_podcast_dto_with_favorites_option
            (&*podcast))
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }

    pub fn get_downloaded_episodes(&mut self) -> Vec<PodcastEpisode> {
        use crate::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        use crate::schema::podcast_episodes::dsl::status as dsl_status;
        dsl_podcast_episodes
            .filter(dsl_status.eq("D"))
            .load::<PodcastEpisode>(&mut self.conn)
            .expect("Error loading podcast episode by id")
    }

    pub fn update_podcast_fields(&mut self, podcast_extra: PodcastExtra) {
        use crate::schema::podcasts::dsl::*;

        do_retry(||{diesel::update(podcasts)
            .filter(id.eq(podcast_extra.clone().id))
            .set((
                author.eq(podcast_extra.clone().author),
                keywords.eq(podcast_extra.clone().keywords),
                explicit.eq(podcast_extra.clone().explicit.to_string()),
                language.eq(podcast_extra.clone().language),
                summary.eq(podcast_extra.clone().description),
                last_build_date.eq(podcast_extra.clone().last_build_date),
            ))
            .execute(&mut self.conn)})
            .expect("Error updating podcast episode");
    }

    pub fn get_settings(&mut self) -> Option<Setting> {
        use crate::schema::settings::dsl::*;

        settings
            .first::<Setting>(&mut self.conn)
            .optional()
            .unwrap()
    }

    pub fn get_podcast_episodes_older_than_days(&mut self, days: i32) -> Vec<PodcastEpisode> {
        use crate::schema::podcast_episodes::dsl::*;

        podcast_episodes
            .filter(download_time.lt(Utc::now().naive_utc() - Duration::days(days as i64)))
            .load::<PodcastEpisode>(&mut self.conn)
            .expect("Error loading podcast episode by id")
    }

    pub fn update_download_status_of_episode(&mut self, id_to_find: i32) {
        use crate::schema::podcast_episodes::dsl::*;
        do_retry(||{diesel::update(podcast_episodes.filter(id.eq(id_to_find)))
            .set((status.eq("N"), download_time.eq(sql("NULL"))))
            .get_result::<PodcastEpisode>(&mut self.conn)}
        ).expect("Error updating podcast episode");
    }

    pub fn update_settings(&mut self, setting: Setting) -> Setting {
        use crate::schema::settings::dsl::*;
        let setting_to_update = settings
            .first::<Setting>(&mut self.conn)
            .expect("Error loading settings");
        do_retry(||{diesel::update(&setting_to_update)
            .set(setting.clone())
            .get_result::<Setting>(&mut self.conn)})
            .expect("Error updating settings")
    }

    pub fn insert_default_settings(&mut self) {
        use crate::schema::settings::dsl::*;

        do_retry(||{insert_into(settings)
            .values(DEFAULT_SETTINGS)
            .execute(&mut self.conn)})
            .expect("Error setting default values");
    }

    pub fn update_podcast_active(conn: &mut SqliteConnection, podcast_id: i32) {
        use crate::schema::podcasts::dsl::*;

        let found_podcast = DB::get_podcast( conn, podcast_id);

        match found_podcast {
            Ok(found_podcast) => {
                do_retry(||{diesel::update(podcasts.filter(id.eq(podcast_id)))
                    .set(active.eq(!found_podcast.active))
                    .execute(conn)})
                    .expect("Error updating podcast episode");
            }
            Err(e) => {
                panic!("Error updating podcast active: {}", e);
            }
        }
    }

    pub fn update_original_image_url(
        &mut self,
        original_image_url_to_set: &str,
        podcast_id_to_find: i32,
    ) {
        use crate::schema::podcasts::dsl::*;
        do_retry(||{ diesel::update(podcasts.filter(id.eq(podcast_id_to_find)))
            .set(original_image_url.eq(original_image_url_to_set))
            .execute(&mut self.conn)})
            .expect("Error updating podcast episode");
    }

    pub fn get_downloaded_episodes_by_podcast_id(
        &mut self,
        id_to_search: i32,
    ) -> Vec<PodcastEpisode> {
        use crate::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(id_to_search))
            .filter(status.eq("D"))
            .load::<PodcastEpisode>(&mut self.conn)
            .expect("Error loading podcast episode by id")
    }


    pub fn get_timeline(username_to_search: String, conn: &mut SqliteConnection) -> Vec<
        (PodcastEpisode, Podcast)> {
        let podcast_timeline = sql_query("SELECT * FROM podcast_episodes,podcasts, \
        favorites \
        WHERE podcasts.id = podcast_episodes.podcast_id AND podcasts.id = favorites.podcast_id \
        AND favorites.username=? AND favored=1 ORDER BY podcast_episodes.date_of_recording DESC \
        LIMIT 20");

        let res = podcast_timeline.bind::<Text, _>(&username_to_search);


        res.load::<(PodcastEpisode, Podcast)>(conn).expect("Error loading podcast episode by id")
    }

    pub fn get_watch_logs_by_username(username_to_search: String, conn: &mut SqliteConnection,
                                      since: NaiveDateTime)
        ->
    Vec<(PodcastHistoryItem, PodcastEpisode, Podcast)> {


        let res = sql_query("SELECT * FROM podcast_history_items,podcast_episodes, podcasts WHERE \
        podcast_history_items.episode_id = podcast_episodes.episode_id AND podcast_history_items\
        .podcast_id= podcasts.id AND username=? AND date >= ?")
            .bind::<Text, _>(&username_to_search)
            .bind::<Timestamp, _>(&since)
            .load::<(PodcastHistoryItem, PodcastEpisode, Podcast)>(conn)
            .expect("Error loading watch logs");

        res
    }

    pub fn get_podcast_by_rss_feed(rss_feed_1:String, conn: &mut SqliteConnection) -> Podcast {
        use crate::schema::podcasts::dsl::*;

        podcasts
            .filter(rssfeed.eq(rss_feed_1))
            .first::<Podcast>(conn)
            .expect("Error loading podcast by rss feed")
    }

    pub fn search_podcasts_favored(conn: &mut SqliteConnection, order: OrderCriteria, title: Option<String>,
                                   latest_pub: OrderOption,
                                   designated_username: String) ->Vec<(Podcast, Favorite)>{
        use crate::schema::podcasts::dsl::*;
        use crate::schema::podcast_episodes::dsl::*;
        use crate::schema::podcasts::dsl::id as podcastsid;
        use crate::schema::favorites;


        let mut query = podcasts.inner_join(podcast_episodes.on(podcastsid.eq(podcast_id)))
            .inner_join(favorites::table.on(podcastsid.eq(favorites::dsl::podcast_id).and
            (favorites::dsl::username.eq(designated_username))))
            .into_boxed();

        match latest_pub {
            OrderOption::Title=> {
                use crate::schema::podcasts::dsl::name as podcasttitle;
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(podcasttitle.asc());
                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(podcasttitle.desc());
                    }
                }
            }
            OrderOption::PublishedDate => {
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(date_of_recording.asc());

                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(date_of_recording.desc());
                    }
                }
            }
        }

        if title.is_some() {
            use crate::schema::podcasts::dsl::name as podcasttitle;
            query = query
                .filter(podcasttitle.like(format!("%{}%", title.unwrap())));
        }

        let mut matching_podcast_ids = vec![];
        let pr = query
            .load::<(Podcast, PodcastEpisode, Favorite)>(conn).expect("Error loading podcasts");
        let distinct_podcasts:Vec<(Podcast, Favorite)> = pr.iter()
            .filter(|c|{
                if matching_podcast_ids.contains(&c.0.id){
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            }).map(|c|{
            (c.clone().0, c.clone().2)
        }).collect::<Vec<(Podcast, Favorite)>>();
        distinct_podcasts
    }

    pub fn search_podcasts(conn: &mut SqliteConnection, order: OrderCriteria, title: Option<String>,
                           latest_pub: OrderOption,
                           designated_username: String) -> Vec<(Podcast, Option<Favorite>)> {
        use crate::schema::podcasts::dsl::*;
        use crate::schema::podcast_episodes::dsl::*;
        use crate::schema::podcasts::dsl::id as podcastsid;
        use crate::schema::favorites::dsl::favorites as f_db;
        use crate::schema::favorites::dsl::podcast_id as f_id;
        use crate::schema::favorites::dsl::username as f_username;

        let mut query = podcasts.inner_join(podcast_episodes.on(podcastsid.eq(podcast_id)))
            .left_join(f_db.on(f_username.eq(designated_username).and(f_id.eq(podcast_id))))
            .into_boxed();

        match latest_pub {
            OrderOption::Title=> {
                use crate::schema::podcasts::dsl::name as podcasttitle;
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(podcasttitle.asc());
                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(podcasttitle.desc());
                    }
                }
            }
            OrderOption::PublishedDate => {
                match order {
                    OrderCriteria::ASC => {
                        query = query.order_by(date_of_recording.asc());

                    }
                    OrderCriteria::DESC => {
                        query = query.order_by(date_of_recording.desc());
                    }
                }
            }
        }

        if title.is_some() {
            use crate::schema::podcasts::dsl::name as podcasttitle;
            query = query
                .filter(podcasttitle.like(format!("%{}%", title.unwrap())));
        }

        let mut matching_podcast_ids = vec![];
        let pr = query
            .load::<(Podcast, PodcastEpisode, Option<Favorite>)>(conn).expect("Error loading \
            podcasts");
        let distinct_podcasts = pr.iter()
            .filter(|c|{
                if matching_podcast_ids.contains(&c.0.id){
                    return false;
                }
                matching_podcast_ids.push(c.0.id);
                true
            }).map(|c|{
            (c.clone().0, c.clone().2)
        }).collect::<Vec<(Podcast, Option<Favorite>)>>();
        distinct_podcasts
    }
}
