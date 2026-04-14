use crate::db::{Database, PersistenceError};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::filter::{Filter, FilterRepository};

diesel::table! {
    filters (user_id) {
        user_id -> Integer,
        title -> Nullable<Text>,
        ascending -> Bool,
        filter -> Nullable<Text>,
        only_favored -> Bool,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = filters)]
struct FilterEntity {
    user_id: i32,
    title: Option<String>,
    ascending: bool,
    filter: Option<String>,
    only_favored: bool,
}

impl From<FilterEntity> for Filter {
    fn from(value: FilterEntity) -> Self {
        Self {
            user_id: value.user_id,
            title: value.title,
            ascending: value.ascending,
            filter: value.filter,
            only_favored: value.only_favored,
        }
    }
}

impl From<Filter> for FilterEntity {
    fn from(value: Filter) -> Self {
        Self {
            user_id: value.user_id,
            title: value.title,
            ascending: value.ascending,
            filter: value.filter,
            only_favored: value.only_favored,
        }
    }
}

pub struct DieselFilterRepository {
    database: Database,
}

impl DieselFilterRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl FilterRepository for DieselFilterRepository {
    type Error = PersistenceError;

    fn get_by_user_id(&self, user_id_to_find: i32) -> Result<Option<Filter>, Self::Error> {
        use self::filters::dsl::*;

        filters
            .filter(user_id.eq(user_id_to_find))
            .first::<FilterEntity>(&mut self.database.connection()?)
            .optional()
            .map(|found_filter| found_filter.map(Into::into))
            .map_err(Into::into)
    }

    fn save(&self, filter_to_save: Filter) -> Result<(), Self::Error> {
        use self::filters::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = FilterEntity::from(filter_to_save);
        let existing = filters
            .filter(user_id.eq(entity.user_id))
            .first::<FilterEntity>(&mut conn)
            .optional()?;

        match existing {
            Some(_) => {
                diesel::update(filters.filter(user_id.eq(entity.user_id)))
                    .set(entity)
                    .execute(&mut conn)?;
            }
            None => {
                diesel::insert_into(filters)
                    .values(entity)
                    .execute(&mut conn)?;
            }
        }
        Ok(())
    }

    fn save_timeline_decision(
        &self,
        user_id_to_update: i32,
        only_favored_to_insert: bool,
    ) -> Result<(), Self::Error> {
        use self::filters::dsl::*;

        diesel::update(filters.filter(user_id.eq(user_id_to_update)))
            .set(only_favored.eq(only_favored_to_insert))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }
}
