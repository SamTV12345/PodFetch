use diesel::{OptionalExtension, RunQueryDsl};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::settings::filter::FilterEntity;
use crate::domain::models::settings::filter::Filter;
use crate::execute_with_conn;
use crate::utils::error::{map_db_error, CustomError};


pub struct FilterRepository;

impl FilterRepository {

    #[allow(clippy::redundant_closure_call)]
    pub fn save_filter(filter_to_insert: Filter) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::filters::dsl::*;

        let opt_filter = filters
            .filter(username.eq(&filter_to_insert.username))
            .first::<FilterEntity>(&mut get_connection())
            .optional()
            .expect("Error connecting to database"); // delete all filters
        match opt_filter {
            Some(_) => {
                diesel::update(filters.filter(username.eq(&filter_to_insert.username)))
                    .set(filter_to_insert)
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
            }
            None => {
                execute_with_conn!(|conn| {
                    diesel::insert_into(filters)
                        .values(filter_to_insert)
                        .execute(conn)
                        .map_err(map_db_error)?;
                    Ok(())
                });
            }
        }
        Ok(())
    }

    pub async fn get_filter_by_username(
        username1: &str,
    ) -> Result<Option<Filter>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::filters::dsl::*;
        filters
            .filter(username.eq(username1))
            .first::<FilterEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|filter| filter.map(|f| f.into()))
    }

    pub fn save_decision_for_timeline(
        username_to_search: &str,
        only_favored_to_insert: bool,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::filters::dsl::*;
        diesel::update(filters.filter(username.eq(username_to_search)))
            .set(only_favored.eq(only_favored_to_insert))
            .execute(&mut get_connection())
            .map_err(map_db_error)
    }
}