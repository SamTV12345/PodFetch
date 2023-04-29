use diesel::prelude::*;
use crate::models::itunes_models::Podcast;
use crate::models::user::User;
use crate::schema::favorites;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Associations, Debug, PartialEq, Serialize, Deserialize, Insertable, Clone)]
#[diesel(belongs_to(Podcast, foreign_key = podcast_id))]
#[diesel(belongs_to(User, foreign_key = username))]
pub struct Favorite{
    pub username: String,
    pub podcast_id: i32,
    pub favored: bool
}

impl Favorite{
    pub fn delete_by_username(username1: String, conn: &mut SqliteConnection) -> Result<(),
        diesel::result::Error>{
        use crate::schema::favorites::dsl::*;
        diesel::delete(favorites.filter(username.eq(username1))).execute(conn)?;
        Ok(())
    }
}