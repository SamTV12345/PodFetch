use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel::associations::HasTable;
use utoipa::ToSchema;
use crate::schema::users;
use diesel::QueryDsl;
use diesel::ExpressionMethods;

#[derive(Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub created_at: NaiveDateTime
}


impl User{
    pub fn new(id: i32, username: String, role: String, password: Option<String>, created_at: NaiveDateTime) -> Self {
        User {
            id,
            username,
            role,
            password,
            created_at
        }
    }

    pub fn insert_user(&mut self, conn: &mut SqliteConnection) -> Result<User, diesel::result::Error> {
        use crate::schema::users::dsl::*;


        self.id = 0;
        diesel::insert_into(users::table())
            .values((
                username.eq(self.username.clone()),
                role.eq(self.role.clone()),
                password.eq(self.password.clone()),
                created_at.eq(chrono::Utc::now().naive_utc())
                ))
            .get_result(conn)
    }

    pub fn delete_user(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(users::table.filter(users::id.eq(self.id)))
            .execute(conn)
    }

    pub fn update_role(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(users::role.eq(self.role.clone()))
            .execute(conn)
    }
}