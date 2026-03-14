use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::model::project::Project;
use crate::mgmt::repository::db::DbPool;
use crate::mgmt::schema::schema::projects;

#[derive(Clone)]
pub struct ProjectRepository {
    pool: DbPool,
}

impl ProjectRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, id: Uuid, org_id: Uuid, name: &str) -> Result<Project> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::insert_into(projects::table)
            .values((
                projects::id.eq(id),
                projects::organization_id.eq(org_id),
                projects::name.eq(name),
            ))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        projects::table
            .filter(projects::id.eq(id))
            .first::<Project>(conn)
            .await
            .optional()
            .map_err(Into::into)
    }

    pub async fn list_by_org_id(&self, org_id: Uuid) -> Result<Vec<Project>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        projects::table
            .filter(projects::organization_id.eq(org_id))
            .load::<Project>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn list_all(&self) -> Result<Vec<Project>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        projects::table
            .load::<Project>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn update(&self, id: Uuid, name: &str) -> Result<Project> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(projects::table.filter(projects::id.eq(id)))
            .set(projects::name.eq(name))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::delete(projects::table.filter(projects::id.eq(id)))
            .execute(conn)
            .await
            .map_err(|_| AppError::Internal("Failed to delete project".into()))?;
        Ok(())
    }
}
