use std::io::Error;
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use diesel::{OptionalExtension, RunQueryDsl, SqliteConnection};
use diesel::associations::HasTable;
use utoipa::ToSchema;
use crate::schema::users;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use dotenv::var;
use sha256::digest;
use crate::constants::constants::{Role, USERNAME};

#[derive(Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub created_at: NaiveDateTime
}


impl User{
    pub fn new(id: i32, username: String, role: Role, password: Option<String>, created_at:
    NaiveDateTime) -> Self {
        User {
            id,
            username,
            role: role.to_string(),
            password,
            created_at
        }
    }

    pub fn find_by_username(username_to_find: &str, conn: &mut SqliteConnection) -> Option<User> {
        use crate::schema::users::dsl::*;

        if var(USERNAME).unwrap()==username_to_find {
            return Some(User::create_admin_user());
        }

        users.filter(username.eq(username_to_find))
            .first::<User>(conn)
            .optional()
            .unwrap()
    }

    pub fn insert_user(&mut self, conn: &mut SqliteConnection) -> Result<User, Error> {
        use crate::schema::users::dsl::*;

        if var(USERNAME).unwrap()==self.username {
        return Err(Error::new(std::io::ErrorKind::Other, "Username already exists"));
        }

        let password_to_insert = digest(self.password.clone().unwrap());
        let res = diesel::insert_into(users::table())
            .values((
                username.eq(self.username.clone()),
                role.eq(self.role.clone()),
                password.eq(password_to_insert),
                created_at.eq(chrono::Utc::now().naive_utc())
                ))
            .get_result::<User>(conn).unwrap();
        Ok(res)
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

    fn create_admin_user()->User{
        User{
            id: 9999,
            username: var(USERNAME).unwrap(),
            role: Role::Admin.to_string(),
            password: None,
            created_at: Default::default(),
        }
    }
}