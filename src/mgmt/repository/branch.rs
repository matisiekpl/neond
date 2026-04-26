use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
use std::collections::HashSet;
use uuid::Uuid;
use crate::mgmt::compute::ComputeEndpointStatus;
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
        slug: &str,
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
                branches::slug.eq(slug),
            ))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::slug.eq(slug))
            .first::<Branch>(conn)
            .await
            .optional()
            .map_err(Into::into)
    }

    pub async fn find_by_project_and_name(
        &self,
        project_id: Uuid,
        name: &str,
    ) -> Result<Option<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::project_id.eq(project_id))
            .filter(branches::name.eq(name))
            .first::<Branch>(conn)
            .await
            .optional()
            .map_err(Into::into)
    }

    pub async fn find_by_timeline_id(&self, timeline_id: Uuid) -> Result<Option<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::timeline_id.eq(timeline_id))
            .first::<Branch>(conn)
            .await
            .optional()
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

    pub async fn list_by_parent_id(&self, parent_id: Uuid) -> Result<Vec<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::parent_branch_id.eq(parent_id))
            .load::<Branch>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn list_by_project_id(&self, project_id: Uuid) -> Result<Vec<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::project_id.eq(project_id))
            .order(branches::created_at.asc())
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

    pub async fn update_password(&self, id: Uuid, password: &str) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(branches::table.filter(branches::id.eq(id)))
            .set(branches::password.eq(password))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn update_recent_status(&self, id: Uuid, status: ComputeEndpointStatus) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(branches::table.filter(branches::id.eq(id)))
            .set(branches::recent_status.eq(status))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn update_port(&self, id: Uuid, port: Option<i32>) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(branches::table.filter(branches::id.eq(id)))
            .set(branches::port.eq(port))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn list_all_with_recent_status(&self, status: ComputeEndpointStatus) -> Result<Vec<Branch>> {
        let conn = &mut self.pool.get().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        branches::table
            .filter(branches::recent_status.eq(status))
            .load::<Branch>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn restore_swap(
        &self,
        old_id: Uuid,
        archive_slug: &str,
        archive_name: &str,
        new_id: Uuid,
        new_slug: &str,
        new_password: &str,
        new_name: &str,
        new_timeline_id: Uuid,
        project_id: Uuid,
        reparented_timeline_ids: &HashSet<Uuid>,
        new_port: Option<i32>,
    ) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;

        let archive_slug = archive_slug.to_string();
        let archive_name = archive_name.to_string();
        let new_slug = new_slug.to_string();
        let new_password = new_password.to_string();
        let new_name = new_name.to_string();
        let reparented_ids: Vec<Uuid> = reparented_timeline_ids.iter().copied().collect();

        conn.transaction::<Branch, AppError, _>(|conn| {
            async move {
                diesel::update(branches::table.filter(branches::id.eq(old_id)))
                    .set((
                        branches::slug.eq(&archive_slug),
                        branches::name.eq(&archive_name),
                        branches::recent_status.eq(ComputeEndpointStatus::Stopped),
                        branches::port.eq(None::<i32>),
                    ))
                    .execute(conn)
                    .await
                    .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;

                let inserted: Branch = diesel::insert_into(branches::table)
                    .values((
                        branches::id.eq(new_id),
                        branches::project_id.eq(project_id),
                        branches::name.eq(&new_name),
                        branches::parent_branch_id.eq(None::<Uuid>),
                        branches::timeline_id.eq(new_timeline_id),
                        branches::password.eq(&new_password),
                        branches::slug.eq(&new_slug),
                        branches::port.eq(new_port),
                    ))
                    .get_result(conn)
                    .await
                    .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;

                if !reparented_ids.is_empty() {
                    diesel::update(
                        branches::table
                            .filter(branches::parent_branch_id.eq(old_id))
                            .filter(branches::id.ne(new_id))
                            .filter(branches::timeline_id.eq_any(&reparented_ids)),
                    )
                    .set(branches::parent_branch_id.eq(new_id))
                    .execute(conn)
                    .await
                    .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;
                }

                Ok(inserted)
            }
            .scope_boxed()
        })
        .await
    }

    pub async fn detach_ancestor_swap(
        &self,
        branch_id: Uuid,
        reparented_timeline_ids: &HashSet<Uuid>,
    ) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|error| AppError::DetachAncestorFailed { reason: error.to_string() })?;

        let reparented_ids: Vec<Uuid> = reparented_timeline_ids.iter().copied().collect();

        conn.transaction::<Branch, AppError, _>(|conn| {
            async move {
                let current: Branch = branches::table
                    .filter(branches::id.eq(branch_id))
                    .first(conn)
                    .await
                    .map_err(|error| match error {
                        diesel::result::Error::NotFound => AppError::NotFound,
                        other => AppError::DetachAncestorFailed { reason: other.to_string() },
                    })?;

                let old_parent_id = current
                    .parent_branch_id
                    .ok_or(AppError::BranchAlreadyDetached)?;

                diesel::update(branches::table.filter(branches::id.eq(branch_id)))
                    .set(branches::parent_branch_id.eq(None::<Uuid>))
                    .execute(conn)
                    .await
                    .map_err(|error| AppError::DetachAncestorFailed { reason: error.to_string() })?;

                if !reparented_ids.is_empty() {
                    diesel::update(
                        branches::table
                            .filter(branches::parent_branch_id.eq(old_parent_id))
                            .filter(branches::id.ne(branch_id))
                            .filter(branches::timeline_id.eq_any(&reparented_ids)),
                    )
                    .set(branches::parent_branch_id.eq(branch_id))
                    .execute(conn)
                    .await
                    .map_err(|error| AppError::DetachAncestorFailed { reason: error.to_string() })?;
                }

                let updated: Branch = branches::table
                    .filter(branches::id.eq(branch_id))
                    .first(conn)
                    .await
                    .map_err(|error| AppError::DetachAncestorFailed { reason: error.to_string() })?;

                Ok(updated)
            }
            .scope_boxed()
        })
        .await
    }

    pub async fn reset_to_parent_swap(
        &self,
        old_id: Uuid,
        archive_slug: &str,
        archive_name: &str,
        new_id: Uuid,
        new_slug: &str,
        new_password: &str,
        new_name: &str,
        new_timeline_id: Uuid,
        parent_branch_id: Uuid,
        project_id: Uuid,
        new_port: Option<i32>,
    ) -> Result<Branch> {
        let conn = &mut self.pool.get().await
            .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;

        let archive_slug = archive_slug.to_string();
        let archive_name = archive_name.to_string();
        let new_slug = new_slug.to_string();
        let new_password = new_password.to_string();
        let new_name = new_name.to_string();

        conn.transaction::<Branch, AppError, _>(|conn| {
            async move {
                diesel::update(branches::table.filter(branches::id.eq(old_id)))
                    .set((
                        branches::slug.eq(&archive_slug),
                        branches::name.eq(&archive_name),
                        branches::recent_status.eq(ComputeEndpointStatus::Stopped),
                        branches::port.eq(None::<i32>),
                    ))
                    .execute(conn)
                    .await
                    .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;

                let inserted: Branch = diesel::insert_into(branches::table)
                    .values((
                        branches::id.eq(new_id),
                        branches::project_id.eq(project_id),
                        branches::name.eq(&new_name),
                        branches::parent_branch_id.eq(Some(parent_branch_id)),
                        branches::timeline_id.eq(new_timeline_id),
                        branches::password.eq(&new_password),
                        branches::slug.eq(&new_slug),
                        branches::port.eq(new_port),
                    ))
                    .get_result(conn)
                    .await
                    .map_err(|error| AppError::PitrSwapFailed { reason: error.to_string() })?;

                Ok(inserted)
            }
            .scope_boxed()
        })
        .await
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
