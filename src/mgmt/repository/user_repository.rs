use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::repository::db::DbPool;
use crate::mgmt::model::user::User;
use crate::mgmt::schema::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    id: Uuid,
    name: &'a str,
    email: &'a str,
    password_hash: &'a str,
}

pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        users::table
            .filter(users::email.eq(email))
            .first::<User>(conn)
            .await
            .optional()
            .map_err(Into::into)
    }

    pub async fn create(&self, id: Uuid, name: &str, email: &str, password_hash: &str) -> Result<User> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let new_user = NewUser { id, name, email, password_hash };
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)
            .await
            .map_err(Into::into)
    }
}
