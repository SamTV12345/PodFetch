use crate::db::{Database, PersistenceError};
use chrono::{NaiveDateTime, Utc};
use diesel::BoolExpressionMethods;
use diesel::JoinOnDsl;
use diesel::OptionalExtension;
use diesel::prelude::{AsChangeset, Insertable, Queryable, QueryableByName};
use diesel::sql_types::{Integer, Nullable, Text, Timestamp};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::subscription::{
    GPodderAvailablePodcast, Subscription, SubscriptionModelChanges, SubscriptionRepository,
};

diesel::table! {
    subscriptions (id) {
        id -> Integer,
        username -> Text,
        device -> Text,
        podcast -> Text,
        created -> Timestamp,
        deleted -> Nullable<Timestamp>,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Integer,
        rssfeed -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(subscriptions, podcasts);

#[derive(Debug, Clone, Queryable, QueryableByName, Insertable, AsChangeset)]
#[diesel(table_name = subscriptions)]
#[diesel(treat_none_as_null = true)]
struct SubscriptionEntity {
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = Text)]
    device: String,
    #[diesel(sql_type = Text)]
    podcast: String,
    #[diesel(sql_type = Timestamp)]
    created: NaiveDateTime,
    #[diesel(sql_type = Nullable<Timestamp>)]
    deleted: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Queryable, QueryableByName)]
struct GPodderAvailablePodcastEntity {
    #[diesel(sql_type = Text)]
    device: String,
    #[diesel(sql_type = Text)]
    podcast: String,
}

impl From<SubscriptionEntity> for Subscription {
    fn from(value: SubscriptionEntity) -> Self {
        Self {
            id: value.id,
            username: value.username,
            device: value.device,
            podcast: value.podcast,
            created: value.created,
            deleted: value.deleted,
        }
    }
}

impl From<GPodderAvailablePodcastEntity> for GPodderAvailablePodcast {
    fn from(value: GPodderAvailablePodcastEntity) -> Self {
        Self {
            device: value.device,
            podcast: value.podcast,
        }
    }
}

pub struct DieselSubscriptionRepository {
    database: Database,
}

impl DieselSubscriptionRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SubscriptionRepository for DieselSubscriptionRepository {
    type Error = PersistenceError;

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        use self::subscriptions::dsl as subscriptions_dsl;

        diesel::delete(
            subscriptions_dsl::subscriptions.filter(subscriptions_dsl::username.eq(username)),
        )
        .execute(&mut self.database.connection()?)
        .map(|_| ())
        .map_err(Into::into)
    }

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        use self::subscriptions::dsl as subscriptions_dsl;

        let subscriptions = subscriptions_dsl::subscriptions
            .filter(subscriptions_dsl::username.eq(username))
            .filter(
                subscriptions_dsl::device
                    .eq(device_id)
                    .and(subscriptions_dsl::created.gt(since)),
            )
            .load::<SubscriptionEntity>(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?
            .into_iter()
            .map(Subscription::from)
            .collect::<Vec<_>>();

        let (remove, add): (Vec<_>, Vec<_>) = subscriptions
            .into_iter()
            .partition(|subscription| subscription.deleted.is_some());

        Ok(SubscriptionModelChanges {
            add,
            remove,
            timestamp,
        })
    }

    fn get_user_subscriptions(
        &self,
        username: &str,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        use self::subscriptions::dsl as subscriptions_dsl;

        let subscriptions = subscriptions_dsl::subscriptions
            .filter(subscriptions_dsl::username.eq(username))
            .filter(subscriptions_dsl::created.gt(since))
            .load::<SubscriptionEntity>(&mut self.database.connection()?)
            .map_err(PersistenceError::from)?
            .into_iter()
            .map(Subscription::from)
            .collect::<Vec<_>>();

        let (remove, add): (Vec<_>, Vec<_>) = subscriptions
            .into_iter()
            .partition(|subscription| subscription.deleted.is_some());

        Ok(SubscriptionModelChanges {
            add,
            remove,
            timestamp,
        })
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        username: &str,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        use self::subscriptions::dsl as subscriptions_dsl;

        let mut rewritten_urls = vec![vec![]];
        let mut connection = self.database.connection()?;

        for podcast in add {
            if !podcast.starts_with("http") && !podcast.starts_with("https") {
                rewritten_urls.push(vec![podcast.to_string(), "".to_string()]);
                continue;
            }

            let existing = subscriptions_dsl::subscriptions
                .filter(
                    subscriptions_dsl::username
                        .eq(username)
                        .and(subscriptions_dsl::device.eq(device_id))
                        .and(subscriptions_dsl::podcast.eq(podcast)),
                )
                .first::<SubscriptionEntity>(&mut connection)
                .optional()?;

            match existing {
                Some(existing) => {
                    diesel::update(
                        subscriptions_dsl::subscriptions
                            .filter(subscriptions_dsl::id.eq(existing.id)),
                    )
                    .set(subscriptions_dsl::deleted.eq(None::<NaiveDateTime>))
                    .execute(&mut connection)?;
                }
                None => {
                    let subscription = Subscription::new(
                        username.to_string(),
                        device_id.to_string(),
                        podcast.to_string(),
                    );
                    diesel::insert_into(subscriptions_dsl::subscriptions)
                        .values(SubscriptionEntity {
                            id: subscription.id,
                            username: subscription.username,
                            device: subscription.device,
                            podcast: subscription.podcast,
                            created: subscription.created,
                            deleted: subscription.deleted,
                        })
                        .execute(&mut connection)?;
                }
            }
        }

        for podcast in remove {
            if !podcast.starts_with("http") && !podcast.starts_with("https") {
                rewritten_urls.push(vec![podcast.to_string(), "".to_string()]);
                continue;
            }

            if let Some(existing) = subscriptions_dsl::subscriptions
                .filter(
                    subscriptions_dsl::username
                        .eq(username)
                        .and(subscriptions_dsl::device.eq(device_id))
                        .and(subscriptions_dsl::podcast.eq(podcast)),
                )
                .first::<SubscriptionEntity>(&mut connection)
                .optional()?
            {
                diesel::update(
                    subscriptions_dsl::subscriptions.filter(subscriptions_dsl::id.eq(existing.id)),
                )
                .set(subscriptions_dsl::deleted.eq(Some(Utc::now().naive_utc())))
                .execute(&mut connection)?;
            }
        }

        Ok(rewritten_urls)
    }

    fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        username: &str,
    ) -> Result<Vec<String>, Self::Error> {
        use self::subscriptions::dsl as subscriptions_dsl;

        subscriptions_dsl::subscriptions
            .filter(
                subscriptions_dsl::username
                    .eq(username)
                    .and(subscriptions_dsl::device.eq(device_id))
                    .and(subscriptions_dsl::deleted.is_null()),
            )
            .select(subscriptions_dsl::podcast)
            .load::<String>(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        use self::podcasts::dsl as podcasts_dsl;
        use self::subscriptions::dsl as subscriptions_dsl;

        subscriptions_dsl::subscriptions
            .left_join(
                podcasts_dsl::podcasts.on(subscriptions_dsl::podcast.eq(podcasts_dsl::rssfeed)),
            )
            .select((subscriptions_dsl::device, subscriptions_dsl::podcast))
            .filter(podcasts_dsl::rssfeed.is_null())
            .filter(subscriptions_dsl::device.ne("webview"))
            .distinct()
            .load::<GPodderAvailablePodcastEntity>(&mut self.database.connection()?)
            .map(|podcasts| {
                podcasts
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<GPodderAvailablePodcast>>()
            })
            .map_err(Into::into)
    }
}
