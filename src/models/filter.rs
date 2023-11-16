use crate::dbconfig::schema::filters;
use crate::utils::error::{map_db_error, CustomError};
use crate::{execute_with_conn, DBType as DbConnection};
use diesel::AsChangeset;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::Queryable;
use diesel::{Insertable, OptionalExtension, RunQueryDsl};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset, Queryable, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub username: String,
    pub title: Option<String>,
    pub ascending: bool,
    pub filter: Option<String>,
    pub only_favored: bool,
}

impl Filter {
    pub fn new(
        username: String,
        title: Option<String>,
        ascending: bool,
        filter: Option<String>,
        only_favored: bool,
    ) -> Self {
        Filter {
            username,
            title,
            ascending,
            filter,
            only_favored,
        }
    }

    #[allow(clippy::redundant_closure_call)]
    pub fn save_filter(self, conn: &mut DbConnection) -> Result<(), CustomError> {
        use crate::dbconfig::schema::filters::dsl::*;

        let opt_filter = filters
            .filter(username.eq(&self.username))
            .first::<Filter>(conn)
            .optional()
            .expect("Error connecting to database"); // delete all filters
        match opt_filter {
            Some(_) => {
                diesel::update(filters.filter(username.eq(&self.clone().username)))
                    .set(self)
                    .execute(conn)
                    .map_err(map_db_error)?;
            }
            None => {
                execute_with_conn!(conn, |conn| {
                    diesel::insert_into(filters)
                        .values(self)
                        .execute(conn)
                        .map_err(map_db_error)?;
                    Ok(())
                });
            }
        }
        Ok(())
    }

    pub async fn get_filter_by_username(
        username1: String,
        conn: &mut DbConnection,
    ) -> Result<Option<Filter>, CustomError> {
        use crate::dbconfig::schema::filters::dsl::*;
        let res = filters
            .filter(username.eq(username1))
            .first::<Filter>(conn)
            .optional()
            .map_err(map_db_error)?;
        Ok(res)
    }

    pub fn save_decision_for_timeline(
        username_to_search: String,
        conn: &mut DbConnection,
        only_favored_to_insert: bool,
    ) {
        use crate::dbconfig::schema::filters::dsl::*;
        diesel::update(filters.filter(username.eq(username_to_search)))
            .set(only_favored.eq(only_favored_to_insert))
            .execute(conn)
            .expect("Error connecting to database");
    }
}
