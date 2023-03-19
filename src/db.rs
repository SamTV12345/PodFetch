use std::time::SystemTime;
use chrono::{DateTime, Utc};
use diesel::{insert_into, RunQueryDsl, sql_query};
use feed_rs::model::Entry;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::models::models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastHistoryItem,
                            PodcastWatchedPostModel, Notification as Notification};
use crate::service::mapping_service::MappingService;
use diesel::prelude::*;
use crate::config::dbconfig::establish_connection;

pub struct DB{
    conn: SqliteConnection,
    mapping_service: MappingService
}

impl Clone for DB{
    fn clone(&self) -> Self {
        DB{
            conn: establish_connection(),
            mapping_service: MappingService::new()
        }
    }
}
impl DB{
    pub fn new() -> Result<DB, String>{
        let conn = establish_connection();
        Ok(DB{conn, mapping_service: MappingService::new()})
    }

    pub fn get_podcasts(&mut self) -> Result<Vec<Podcast>, String>{
        use crate::schema::podcasts::dsl::podcasts;
        let result = podcasts
            .load::<Podcast>(&mut self.conn)
            .expect("Error loading podcasts");
        Ok(result)
    }

    pub fn get_podcast(&mut self, podcast_id_to_be_found: i32) -> Result<Podcast, String>{
        use crate::schema::podcasts::{id as podcast_id};
        use crate::schema::podcasts::dsl::podcasts;
        let found_podcast = podcasts
            .filter(podcast_id.eq(podcast_id_to_be_found))
            .first::<Podcast>(&mut self.conn)
            .expect("Error loading podcast by id");

        Ok(found_podcast)
    }

    pub fn get_podcast_episode_by_id(&mut self, podcas_episode_id_to_be_found: &str) ->
                                                                   Result<Option<PodcastEpisode>, String>{
        use crate::schema::podcast_episodes::{episode_id};
        use crate::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(episode_id.eq(podcas_episode_id_to_be_found))
            .first::<PodcastEpisode>(&mut self.conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode)
    }

    pub fn get_podcast_episode_by_url(&mut self, podcas_episode_url_to_be_found: &str) ->
                                                                   Result<Option<PodcastEpisode>, String>{
        use crate::schema::podcast_episodes::{url};
        use crate::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(url.eq(podcas_episode_url_to_be_found))
            .first::<PodcastEpisode>(&mut self.conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode)
    }


    pub fn get_podcast_by_track_id(&mut self, podcast_id: i32) ->
                                                                   Result<Option<Podcast>, String>{
        use crate::schema::podcasts::{directory};
        use crate::schema::podcasts::dsl::podcasts;
        let optional_podcast = podcasts
            .filter(directory.eq(podcast_id.to_string()))
            .first::<Podcast>(&mut self.conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(optional_podcast)
    }

    pub fn insert_podcast_episodes(&mut self, podcast: Podcast, link: &str, item: &Entry,
                                   image_url_1: &str, episode_description: &str,
                                   total_time_of_podcast: i32)->PodcastEpisode{
        use crate::schema::podcast_episodes::dsl::*;
        let uuid_podcast = uuid::Uuid::new_v4();

        let inserted_podcast = insert_into(podcast_episodes)
            .values((
                total_time.eq(total_time_of_podcast),
                podcast_id.eq(podcast.id),
                episode_id.eq(uuid_podcast.to_string()),
                name.eq(item.title.as_ref().unwrap().clone().content),
                url.eq(link.to_string()),
                date_of_recording.eq(&item.published.unwrap().to_rfc3339()),
                image_url.eq(image_url_1.to_string()),
                description.eq(episode_description)
            ))
            .get_result::<PodcastEpisode>(&mut self.conn)
            .expect("Error inserting podcast episode");
        return inserted_podcast;
    }

    pub fn add_podcast_to_database(&mut self, collection_name:String, collection_id:String,
                                   feed_url:String, image_url_1: String)->Podcast{
        use crate::schema::podcasts::{directory, rssfeed, name as podcast_name, image_url};
        use crate::schema::podcasts;

        let inserted_podcast = insert_into(podcasts::table)
            .values((
                directory.eq(collection_id.to_string()),
                podcast_name.eq(collection_name.to_string()),
                rssfeed.eq(feed_url.to_string()),
                image_url.eq(image_url_1.to_string())
            ))
            .get_result::<Podcast>(&mut self.conn)
            .expect("Error inserting podcast");
        return inserted_podcast;
    }

    pub fn get_last_5_podcast_episodes(&mut self, podcast_episode_id: i32) ->
                                                                      Result<Vec<PodcastEpisode>,
                                                                          String>{
        use crate::schema::podcast_episodes::dsl::podcast_episodes as podcast_episodes;
        use crate::schema::podcast_episodes::{date_of_recording, podcast_id};
        let podcasts = podcast_episodes
            .filter(podcast_id.eq(podcast_episode_id))
            .limit(5)
            .order(date_of_recording.desc())
            .load::<PodcastEpisode>(&mut self.conn)
            .expect("Error loading podcasts");
        Ok(podcasts)
    }


    pub fn get_podcast_episodes_of_podcast(&mut self, podcast_id_to_be_searched: i32, last_id:
    Option<String>) ->
                                                                      Result<Vec<PodcastEpisode>, String>{
        use crate::schema::podcast_episodes::dsl::podcast_episodes as podcast_episodes;
        use crate::schema::podcast_episodes::*;
        match last_id {
            Some(last_id) => {
                let podcasts_found = podcast_episodes.filter(podcast_id.eq(podcast_id_to_be_searched))
                    .filter(date_of_recording.lt(last_id))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(&mut self.conn)
                    .expect("Error loading podcasts");
                Ok(podcasts_found)
            }
            None => {
                let podcasts_found = podcast_episodes.filter(podcast_id.eq(podcast_id_to_be_searched))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(&mut self.conn)
                    .expect("Error loading podcasts");

                Ok(podcasts_found)
            }
        }


    }

    pub fn log_watchtime(&mut self, watch_model: PodcastWatchedPostModel) ->Result<(), String> {
        let result = self.get_podcast_episode_by_id(&watch_model.podcast_episode_id).unwrap();

        use crate::schema::podcast_history_items::dsl::*;
        match result {
            Some(result)=>{
                let now = SystemTime::now();
                let now: DateTime<Utc> = now.into();
                let now: &str = &now.to_rfc3339();
                insert_into(podcast_history_items)
                    .values((
                        podcast_id.eq(result.podcast_id),
                        episode_id.eq(result.episode_id),
                        watched_time.eq(watch_model.time),
                        date.eq(&now),
                    ))
                    .execute(&mut self.conn)
                    .expect("Error inserting podcast episode");
                Ok(())
            }
            None=>{
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn get_watchtime(&mut self, podcast_id_tos_search: &str) ->Result<PodcastHistoryItem,
        String>{
        let result = self.get_podcast_episode_by_id(podcast_id_tos_search).unwrap();
        use crate::schema::podcast_history_items::dsl::*;

        match result {
            Some(found_podcast)=>{
                let history_item = podcast_history_items
                    .filter(episode_id.eq(podcast_id_tos_search))
                    .order(date.desc())
                    .first::<PodcastHistoryItem>(&mut self.conn)
                    .optional()
                    .expect("Error loading podcast episode by id");
                return match history_item {
                    Some(found_history_item) => {
                        Ok(found_history_item)
                    }
                    None => {
                        Ok(PodcastHistoryItem {
                            id: 0,
                            podcast_id: found_podcast.podcast_id,
                            episode_id: found_podcast.episode_id,
                            watched_time: 0,
                            date: "".to_string(),
                        })
                    }
                }
            }
            None=>{
                panic!("Podcast episode not found");
            }
        }
    }


    pub fn get_last_watched_podcasts(&mut self)
        -> Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>, String> {

        let result = sql_query("SELECT * FROM (SELECT * FROM podcast_history_items ORDER BY \
        datetime\
        (date) \
        DESC) GROUP BY episode_id  LIMIT 10;")
            .load::<PodcastHistoryItem>(&mut self.conn)
            .unwrap();

        let podcast_watch_episode = result.iter().map(|podcast_watch_model|{
            let optional_podcast = self.get_podcast_episode_by_id(&podcast_watch_model.episode_id)
                .unwrap();
        match optional_podcast {
            Some(podcast_episode) => {

                let podcast_dto = self.mapping_service.map_podcastepisode_to_dto(&podcast_episode);
                let podcast = self.get_podcast(podcast_episode.podcast_id).unwrap();
                let podcast_watch_model = self.mapping_service
                    .map_podcast_history_item_to_with_podcast_episode(&podcast_watch_model.clone(),
                                                                      podcast_dto, podcast);
                return podcast_watch_model
            }
            None => {
                panic!("Podcast episode not found");
            }
        }

    }).collect::<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>();
        Ok(podcast_watch_episode)
    }

    pub fn update_total_podcast_time_and_image(&mut self, episode_id: &str, image_url:
    &str, local_download_url: &str ) -> Result<(), String> {
        use crate::schema::podcast_episodes::dsl::episode_id as episode_id_column;
        use crate::schema::podcast_episodes::dsl::local_image_url as local_image_url_column;
        use crate::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::schema::podcast_episodes::dsl::podcast_episodes as podcast_episodes;
        let result = podcast_episodes
            .filter(episode_id_column.eq(episode_id))
            .first::<PodcastEpisode>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");

        match result {
            Some(..)=>{
                diesel::update(podcast_episodes)
                    .filter(episode_id_column.eq(episode_id))
                    .set((
                        local_image_url_column.eq(image_url),
                        local_url_column.eq(local_download_url)
                    ))
                    .execute(&mut self.conn)
                    .expect("Error updating local image url");
                Ok(())
            }
            None=>{
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn update_podcast_image(mut self, id: &str, image_url: &str)
        -> Result<(), String> {
        use crate::schema::podcasts::dsl::image_url as image_url_column;
        use crate::schema::podcasts::dsl::directory;
        use crate::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result = dsl_podcast
            .filter(directory.eq(id))
            .first::<Podcast>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(..)=>{
                diesel::update(dsl_podcast.filter(directory.eq(id)))
                    .set(image_url_column.eq(image_url))
                    .execute(&mut self.conn)
                    .expect("Error updating podcast episode");
                Ok(())
            }
            None=>{
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn get_podcast_by_directory(mut self, podcast_id: &str)
        ->Result<Option<Podcast>, String>{
        use crate::schema::podcasts::dsl::directory;
        use crate::schema::podcasts::dsl::podcasts as dsl_podcast;
        let result = dsl_podcast
            .filter(directory.eq(podcast_id))
            .first::<Podcast>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");
        Ok(result)

    }

    pub fn check_if_downloaded(&mut self, download_episode_url: &str) ->Result<bool, String>{
        use crate::schema::podcast_episodes::url as podcast_episode_url;
        use crate::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        let result = dsl_podcast_episodes
            .filter(local_url_column.is_not_null())
            .filter(podcast_episode_url.eq(download_episode_url))
            .first::<PodcastEpisode>(&mut self.conn)
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(podcast_episode)=>{
                return match podcast_episode.status.as_str() {
                    "N"=> Ok(false),
                    "D"=> Ok(true),
                    "P"=> Ok(false),
                    _=> Ok(false)
                }
            }
            None=>{
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn update_podcast_episode_status(&mut self, download_url_of_episode: &str, status: &str)
                                         ->Result<PodcastEpisode, String> {
        use crate::schema::podcast_episodes::dsl::status as status_column;
        use crate::schema::podcast_episodes::dsl::url as download_url;
        use crate::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        let updated_podcast = diesel::update(dsl_podcast_episodes.filter(download_url.eq
        (download_url_of_episode)))
                .set(status_column.eq(status))
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
        use crate::schema::notifications::dsl::notifications as notifications;
        use crate::schema::notifications::type_of_message;
        use crate::schema::notifications::*;
        insert_into(notifications)
            .values(
                (
                    type_of_message.eq(notification.type_of_message),
                    message.eq(notification.message),
                    status.eq(notification.status),
                    created_at.eq(notification.created_at),
                    )
            )
            .execute(&mut self.conn)
            .expect("Error inserting Notification");
        Ok(())
    }

    pub fn update_status_of_notification(&mut self, id_to_search: i32, status_update: &str) ->
                                                                                        Result<(),
        String> {
        use crate::schema::notifications::dsl::*;
        diesel::update(notifications.filter(id.eq(id_to_search)))
            .set(status.eq(status_update))
            .execute(&mut self.conn)
            .expect("Error updating notification");
        Ok(())
    }


    pub fn query_for_podcast(&mut self, query: &str) -> Result<Vec<PodcastEpisode>, String> {
        use crate::schema::podcast_episodes::dsl::*;
        let result = podcast_episodes
            .filter(name.like(format!("%{}%", query))
                .or(description.like(format!("%{}%", query))))
            .load::<PodcastEpisode>(&mut self.conn)
            .expect("Error loading podcast episode by id");
        Ok(result)
    }
}
