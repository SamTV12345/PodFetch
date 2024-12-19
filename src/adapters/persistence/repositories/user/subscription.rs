use diesel::{BoolExpressionMethods, RunQueryDsl};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::utils::error::{map_db_error, CustomError};

pub struct SubscriptionRepository;


impl SubscriptionRepository {
    pub fn delete_by_username(username1: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl::*;
        diesel::delete(subscriptions.filter(username.eq(username1)))
            .execute(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn find_by_podcast(
        username_to_find: &str,
        device_to_find: &str,
        podcast_to_find: &str
    ) -> Result<> {
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl::*;
        let res = subscriptions
            .filter(
                username
                    .eq(username_to_find)
                    .and(device.eq(device_to_find))
                    .and(podcast.eq(podcast_to_find)),
            )
            .first::<SubscriptionE>(&mut get_connection())
            .optional()
            .expect("Error retrieving subscription");
    }
}