use crate::DBType as DbConnection;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::subscriptions;
use crate::gpodder::subscription::subscriptions::SubscriptionUpdateRequest;
use crate::utils::error::ErrorSeverity::Critical;
use crate::utils::error::{CustomError, map_db_error};
use crate::utils::time::get_current_timestamp;
use axum::Json;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::sql_types::{Integer, Nullable, Text, Timestamp};
use diesel::{AsChangeset, Insertable, Queryable, QueryableByName};
use diesel::{BoolExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::io::Error;
use utoipa::ToSchema;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    QueryableByName,
    Queryable,
    AsChangeset,
    Insertable,
    Clone,
    ToSchema,
)]
#[diesel(treat_none_as_null = true)]
pub struct Subscription {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String,
    #[diesel(sql_type = Timestamp)]
    pub created: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub deleted: Option<NaiveDateTime>,
}

impl Subscription {
    pub fn new(username: String, device: String, podcast: String) -> Self {
        Self {
            id: 0,
            username,
            device,
            podcast,
            created: Utc::now().naive_utc(),
            deleted: None,
        }
    }
    pub fn delete_by_username(username1: &str, conn: &mut DbConnection) -> Result<(), Error> {
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl::*;
        diesel::delete(subscriptions.filter(username.eq(username1)))
            .execute(conn)
            .expect("Error deleting subscriptions of user");
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct SubscriptionChangesToClient {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}

pub struct SubscriptionModelChangesToClient {
    pub add: Vec<Subscription>,
    pub remove: Vec<Subscription>,
    pub timestamp: i64,
}

impl From<SubscriptionModelChangesToClient> for SubscriptionChangesToClient {
    fn from(value: SubscriptionModelChangesToClient) -> Self {
        Self {
            add: value.add.iter().map(|c| c.podcast.clone()).collect(),
            remove: value.remove.iter().map(|c| c.podcast.clone()).collect(),
            timestamp: value.timestamp,
        }
    }
}

impl SubscriptionChangesToClient {
    pub async fn get_device_subscriptions(
        device_id: &str,
        username: &str,
        since: i32,
    ) -> Result<SubscriptionModelChangesToClient, Error> {
        let since = DateTime::from_timestamp(since as i64, 0)
            .map(|v| v.naive_utc())
            .unwrap();
        let res: Vec<Subscription> = subscriptions::table
            .filter(subscriptions::username.eq(username))
            .filter(
                subscriptions::device
                    .eq(device_id)
                    .and(subscriptions::created.gt(since)),
            )
            .load::<Subscription>(&mut get_connection())
            .expect("Error retrieving changed subscriptions");

        let (deleted_subscriptions, created_subscriptions): (Vec<Subscription>, Vec<Subscription>) =
            res.into_iter().partition(|c| c.deleted.is_some());

        Ok(SubscriptionModelChangesToClient {
            add: created_subscriptions,
            remove: deleted_subscriptions,
            timestamp: get_current_timestamp(),
        })
    }

    pub async fn get_user_subscriptions(
        username: &str,
        since: i32,
    ) -> Result<SubscriptionModelChangesToClient, CustomError> {
        let since = DateTime::from_timestamp(since as i64, 0)
            .map(|v| v.naive_utc())
            .unwrap();
        let res: Vec<Subscription> = subscriptions::table
            .filter(subscriptions::username.eq(username))
            .filter(subscriptions::created.gt(since))
            .load::<Subscription>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        let (deleted_subscriptions, created_subscriptions): (Vec<Subscription>, Vec<Subscription>) =
            res.into_iter().partition(|c| c.deleted.is_some());

        Ok(SubscriptionModelChangesToClient {
            add: created_subscriptions,
            remove: deleted_subscriptions,
            timestamp: get_current_timestamp(),
        })
    }

    pub async fn update_subscriptions(
        device_id: &str,
        username: &str,
        upload_request: Json<SubscriptionUpdateRequest>,
    ) -> Result<Vec<Vec<String>>, Error> {
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl as dsl_types;
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl::subscriptions;
        let mut rewritten_urls: Vec<Vec<String>> = vec![vec![]];
        // Add subscriptions
        upload_request.clone().add.iter().for_each(|c| {
            if !c.starts_with("http") || !c.starts_with("https") {
                rewritten_urls.push(vec![c.to_string(), "".to_string()]);
                return;
            }

            let opt_sub =
                Self::find_by_podcast(username.to_string(), device_id.to_string(), c.to_string())
                    .expect(
                        "Error retrieving \
                                             subscription",
                    );
            match opt_sub {
                Some(s) => {
                    diesel::update(subscriptions.filter(dsl_types::id.eq(s.id)))
                        .set(dsl_types::deleted.eq(None::<NaiveDateTime>))
                        .execute(&mut get_connection())
                        .unwrap();
                }
                None => {
                    let subscription = Subscription::new(
                        username.to_string(),
                        device_id.to_string(),
                        c.to_string(),
                    );
                    diesel::insert_into(subscriptions)
                        .values((
                            dsl_types::username.eq(subscription.username),
                            dsl_types::device.eq(subscription.device),
                            dsl_types::podcast.eq(subscription.podcast),
                            dsl_types::created.eq(subscription.created),
                        ))
                        .execute(&mut get_connection())
                        .unwrap();
                }
            }
        });
        upload_request.clone().remove.iter().for_each(|c| {
            if !c.starts_with("http") || !c.starts_with("https") {
                rewritten_urls.push(vec![c.to_string(), "".to_string()]);
                return;
            }
            let opt_sub =
                Self::find_by_podcast(username.to_string(), device_id.to_string(), c.to_string())
                    .expect(
                        "Error retrieving \
                                             subscription",
                    );
            if let Some(s) = opt_sub {
                diesel::update(subscriptions.filter(dsl_types::id.eq(s.id)))
                    .set(dsl_types::deleted.eq(Some(Utc::now().naive_utc())))
                    .execute(&mut get_connection())
                    .unwrap();
            }
        });

        Ok(rewritten_urls)
    }

    pub fn find_by_podcast(
        username_1: String,
        deviceid_1: String,
        podcast_1: String,
    ) -> Result<Option<Subscription>, Error> {
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl::*;

        let res = subscriptions
            .filter(
                username
                    .eq(username_1)
                    .and(device.eq(deviceid_1))
                    .and(podcast.eq(podcast_1)),
            )
            .first::<Subscription>(&mut get_connection())
            .optional()
            .expect("Error retrieving subscription");

        Ok(res)
    }
}
