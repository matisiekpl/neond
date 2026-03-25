use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::model::branch::Branch;
use crate::mgmt::repository::db::DbPool;
use crate::mgmt::schema::schema::branches;

#[derive(Clone)]
pub struct BranchRepository {
    pool: DbPool,
}

impl BranchRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: Uuid,
        project_id: Uuid,
        name: &str,
        parent_branch_id: Option<Uuid>,
        timeline_id: Uuid,
        password: &str,
    ) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::insert_into(branches::table)
            .values((
                branches::id.eq(id),
                branches::project_id.eq(project_id),
                branches::name.eq(name),
                branches::parent_branch_id.eq(parent_branch_id),
                branches::timeline_id.eq(timeline_id),
                branches::password.eq(password),
            ))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::id.eq(id))
            .first::<Branch>(conn)
            .await
            .optional()
            .map_err(Into::into)
    }

    pub async fn list_by_project_id(&self, project_id: Uuid) -> Result<Vec<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::project_id.eq(project_id))
            .load::<Branch>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn update(&self, id: Uuid, name: &str) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(branches::table.filter(branches::id.eq(id)))
            .set(branches::name.eq(name))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::delete(branches::table.filter(branches::id.eq(id)))
            .execute(conn)
            .await
            .map_err(|_| AppError::Internal("Failed to delete branch".into()))?;
        Ok(())
    }
}
