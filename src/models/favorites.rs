use diesel::prelude::*;
use crate::models::itunes_models::Podcast;
use crate::models::user::User;
use crate::schema::favorites;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Associations, Debug, PartialEq, Serialize, Deserialize, Insertable, Clone)]
#[belongs_to(Podcast, foreign_key = "podcast_id")]
#[belongs_to(User, foreign_key="username")]
pub struct Favorite{
    pub username: String,
    pub podcast_id: i32,
    pub favored: bool
}