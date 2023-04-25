use std::io::Error;
use actix_web::web;
use chrono::{NaiveDateTime, Utc};
use diesel::{BoolExpressionMethods, QueryDsl, RunQueryDsl, sql_query, SqliteConnection};
use crate::models::itunes_models::Podcast;
use crate::gpodder::subscription::subscriptions::SubscriptionUpdateRequest;
use diesel::ExpressionMethods;

use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use diesel::{Insertable, Queryable, QueryableByName, AsChangeset};
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use crate::schema::subscriptions;
use crate::utils::time::get_current_timestamp;

#[derive(Debug, Serialize, Deserialize,QueryableByName, Queryable,AsChangeset,Insertable, Clone,
ToSchema)]
#[changeset_options(treat_none_as_null="true")]
pub struct Subscription{
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device:String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Timestamp)]
    pub created: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub deleted: Option<NaiveDateTime>
}

impl Subscription{
    pub fn new(username: String, device: String, podcast_id: i32) -> Self{
        Self{
            username,
            device,
            podcast_id,
            created: Utc::now().naive_utc(),
            deleted: None
        }
    }
}


#[derive(Debug, Serialize)]
pub struct SubscriptionChangesToClient {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}


impl SubscriptionChangesToClient {
    pub async fn get_device_subscriptions(device_id: &str, username: &str, since: i32,
                                          conn: &mut SqliteConnection) -> Result<SubscriptionChangesToClient, Error>{
        let since = NaiveDateTime::from_timestamp_opt(since as i64, 0).unwrap();
      let res = sql_query("SELECT * FROM subscriptions INNER JOIN podcasts ON subscriptions\
      .podcast_id = podcasts.id WHERE subscriptions.device = ? AND (subscriptions.created > ? OR \
      subscriptions.deleted > ?) AND username = ?")
          .bind::<Text, _>(device_id)
          .bind::<Timestamp, _>(since)
          .bind::<Text, _>(username)
          .load::<(Subscription, Podcast)>(conn)
          .expect("Error loading subscriptions");

      let (deleted_subscriptions,created_subscriptions):(Vec<(Subscription, Podcast)>,
                                                        Vec<(Subscription, Podcast)> ) = res
          .into_iter()
          .partition(|c| c.0.deleted.is_none());

        Ok(SubscriptionChangesToClient{
            add: created_subscriptions.into_iter().map(|c| c.1.rssfeed).collect(),
            remove: deleted_subscriptions.into_iter().map(|c| c.1.rssfeed).collect(),
            timestamp: get_current_timestamp()
        })
    }

    pub async fn update_subscriptions(device_id: &str, username: &str, upload_request:
    web::Json<SubscriptionUpdateRequest>, conn: &mut SqliteConnection)-> Result<Vec<String>, Error>{
        use crate::schema::subscriptions::dsl as dsl_types;
        use crate::schema::subscriptions::dsl::subscriptions;
        println!("Update:{:?}", upload_request.0);
        let res = sql_query("SELECT * FROM subscriptions INNER JOIN podcasts ON subscriptions\
      .podcast_id = podcasts.id WHERE subscriptions.device = ? AND username = ?")
            .bind::<Text, _>(device_id)
            .bind::<Text, _>(username)
            .load::<(Subscription, Podcast)>(conn).unwrap();

        // Add subscriptions
        upload_request.clone().add.iter().for_each(|c| {
            let podcast = Podcast::get_by_rss_feed(c, conn);
            if podcast.is_err() {
                return;
            }

            let podcast = podcast.unwrap();
            let subscription = Subscription::new(username.to_string(), device_id.to_string(), podcast.id);

            let option_sub = res.iter().find(|&x| x.0.username == subscription.username&& x.0.device == subscription.device && x.0.podcast_id == subscription.podcast_id);
            match option_sub {
                Some(_) => {
                    diesel::update(subscriptions.filter(dsl_types::username.eq(&subscription
                    .username).and(dsl_types::device.eq(&subscription.device)).and
                    (dsl_types::podcast_id.eq(&subscription.podcast_id))))
                        .set(dsl_types::deleted.eq(None::<NaiveDateTime>))
                        .execute(conn).unwrap();
                },
                None => {
                    diesel::insert_into(subscriptions).values(&subscription).execute(conn)
                        .unwrap();
                }
            }
        });
        upload_request.clone().remove.iter().for_each(|c|{
            let podcast = Podcast::get_by_rss_feed(c, conn).unwrap();
            let subscription = Subscription::new(username.to_string(), device_id.to_string(), podcast.id);

            let option_sub = res.iter().find(|&x| x.0.username == subscription.username&& x.0.device == subscription.device && x.0.podcast_id == subscription.podcast_id);

            match option_sub {
                Some(_) => {
                    diesel::update(subscriptions.filter(dsl_types::username.eq(&subscription
                    .username).and(dsl_types::device.eq(&subscription.device)).and
                    (dsl_types::podcast_id.eq(&subscription.podcast_id))))
                        .set(dsl_types::deleted.eq(Some(Utc::now().naive_utc())))
                        .execute(conn).unwrap();
                },
                None => {
                    diesel::insert_into(subscriptions).values(&subscription).execute(conn)
                        .unwrap();
                }
            }
        });

        Ok(upload_request.clone().add)
    }
}