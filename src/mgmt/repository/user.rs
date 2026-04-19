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
    is_admin: bool,
}

#[derive(Clone)]
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        users::table
            .filter(users::id.eq(id))
            .first::<User>(conn)
            .await
            .optional()
            .map_err(Into::into)
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

    pub async fn create(&self, id: Uuid, name: &str, email: &str, password_hash: &str, is_admin: bool) -> Result<User> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let new_user = NewUser { id, name, email, password_hash, is_admin };
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn count(&self) -> Result<i64> {
        use diesel::dsl::count_star;
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        users::table
            .select(count_star())
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn find_all(&self) -> Result<Vec<User>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        users::table
            .order(users::created_at.asc())
            .load::<User>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn update(&self, id: Uuid, name: &str, email: &str, is_admin: bool) -> Result<User> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                users::name.eq(name),
                users::email.eq(email),
                users::is_admin.eq(is_admin),
            ))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(users::table.filter(users::id.eq(id)))
            .set(users::password_hash.eq(password_hash))
            .execute(conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::delete(users::table.filter(users::id.eq(id)))
            .execute(conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn exists(&self, id: Uuid) -> Result<bool> {
        use diesel::dsl::exists;
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::select(exists(
            users::table.filter(users::id.eq(id))
        ))
        .get_result(conn)
        .await
        .map_err(Into::into)
    }
}
