use diesel::prelude::*;
use crate::models::itunes_models::Podcast;
use crate::models::user::User;
use crate::dbconfig::schema::favorites;
use serde::{Serialize, Deserialize};
use diesel::sql_types::{Text, Integer, Bool};

#[derive(Queryable, Associations, Debug, PartialEq,QueryableByName, Serialize, Deserialize, Insertable,
Clone,
AsChangeset)]
#[diesel(belongs_to(Podcast, foreign_key = podcast_id))]
#[diesel(belongs_to(User, foreign_key = username))]
pub struct Favorite{
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Bool)]
    pub favored: bool
}

impl Favorite{
    pub fn delete_by_username(username1: String, conn: &mut SqliteConnection) -> Result<(),
        diesel::result::Error>{
        use crate::dbconfig::schema::favorites::dsl::*;
        diesel::delete(favorites.filter(username.eq(username1))).execute(conn)?;
        Ok(())
    }
}