use diesel::{Insertable, OptionalExtension, RunQueryDsl, SqliteConnection};
use crate::schema::filters;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use diesel::AsChangeset;
use diesel::Queryable;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset, Queryable)]
pub struct Filter{
    username: String,
    title: Option<String>,
    ascending: bool,
    filter: Option<String>
}


impl Filter{
    pub fn new(username: String, title: Option<String>, ascending: bool, filter: Option<String>) -> Self{
        Filter{
            username,
            title,
            ascending,
            filter
        }
    }

    pub fn save_filter(self, conn: &mut SqliteConnection) -> Result<(), diesel::result::Error>{
        use crate::schema::filters::dsl::*;

        let opt_filter = filters.filter(username.eq(&self.username)).first::<Filter>(conn)
            .optional().expect("Error connecting to database"); // delete all filters
       match opt_filter {
           Some(_)=>{
               diesel::update(filters.filter(username.eq(&self.clone().username))).set(self)
                   .execute(conn)?;
           },
           None=>{
               diesel::insert_into(filters).values(self)
                   .execute(conn)?;
           }
       }
        Ok(())
    }

    pub fn get_filter_by_username(username1: String, conn: &mut SqliteConnection) -> Result<Option<Filter>, diesel::result::Error>{
        use crate::schema::filters::dsl::*;
        let opt_filter = filters.filter(username.eq(username1)).first::<Filter>(conn)
            .optional().expect("Error connecting to database"); // delete all filters
        Ok(opt_filter)
    }
}