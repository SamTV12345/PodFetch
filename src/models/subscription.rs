use std::io::Error;
use std::ops::Deref;
use chrono::{NaiveDateTime, Utc};
use diesel::{QueryDsl, RunQueryDsl, sql_query, SqliteConnection};
use crate::models::itunes_models::Podcast;
use crate::schema::subscriptions::dsl::subscriptions;
use crate::service::subscription::Subscription;

#[derive(Debug, Serialize)]
pub struct SubscriptionChangesToClient {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: NaiveDateTime,
}


impl SubscriptionChangesToClient {
    pub async fn get_device_subscriptions(device_id: &str, username: &str, since: NaiveDateTime,
                                          conn: &mut SqliteConnection)-> Result<SubscriptionChangesToClient, Error>{

      let res = sql_query("SELECT * FROM subscriptions INNER JOIN podcasts ON subscriptions\
      .podcast_id = podcasts.id WHERE subscriptions.device = ? AND (subscriptions.created > ? OR \
      subscriptions.deleted > ? AND username = ?")
          .bind::<diesel::sql_types::Text, _>(device_id)
          .bind::<diesel::sql_types::Timestamp, _>(since)
          .bind::<diesel::sql_types::Text, _>(username)
          .load::<(Subscription, Podcast)>(conn)
          .expect("Error loading subscriptions");

      let (deleted_subscriptions,created_subscriptions):(Vec<(Subscription, Podcast)>,
                                                        Vec<(Subscription, Podcast)> ) = res
          .into_iter()
          .partition(|c| c.0.deleted.is_none());

        Ok(SubscriptionChangesToClient{
            add: created_subscriptions.into_iter().map(|c| c.1.rssfeed).collect(),
            remove: deleted_subscriptions.into_iter().map(|c| c.1.rssfeed).collect(),
            timestamp: Utc::now().naive_utc()
        })
    }
}