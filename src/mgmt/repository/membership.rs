use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::model::membership::Membership;
use crate::mgmt::repository::db::DbPool;
use crate::mgmt::schema::schema::memberships;

#[derive(Clone)]
pub struct MembershipRepository {
    pool: DbPool,
}

impl MembershipRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: Uuid, organization_id: Uuid) -> Result<Membership> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::insert_into(memberships::table)
            .values((
                memberships::user_id.eq(user_id),
                memberships::organization_id.eq(organization_id),
            ))
            .get_result(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<Membership>> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        memberships::table
            .filter(memberships::user_id.eq(user_id))
            .load::<Membership>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn list_by_org_id(&self, org_id: Uuid) -> Result<Vec<Membership>> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        memberships::table
            .filter(memberships::organization_id.eq(org_id))
            .load::<Membership>(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(&self, user_id: Uuid, org_id: Uuid) -> Result<()> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::delete(
            memberships::table
                .filter(memberships::user_id.eq(user_id))
                .filter(memberships::organization_id.eq(org_id)),
        )
        .execute(conn)
        .await
        .map_err(|_| AppError::Internal("Failed to delete membership".into()))?;
        Ok(())
    }

    pub async fn delete_by_org_id(&self, org_id: Uuid) -> Result<()> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::delete(memberships::table.filter(memberships::organization_id.eq(org_id)))
            .execute(conn)
            .await
            .map_err(|_| AppError::Internal("Failed to delete memberships".into()))?;
        Ok(())
    }

    pub async fn delete_by_user_id(&self, user_id: Uuid) -> Result<()> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::delete(memberships::table.filter(memberships::user_id.eq(user_id)))
            .execute(conn)
            .await
            .map_err(|_| AppError::Internal("Failed to delete memberships".into()))?;
        Ok(())
    }

    pub async fn exists(&self, user_id: Uuid, org_id: Uuid) -> Result<bool> {
        use diesel::dsl::exists;
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::select(exists(
            memberships::table
                .filter(memberships::user_id.eq(user_id))
                .filter(memberships::organization_id.eq(org_id))
        ))
        .get_result(conn)
        .await
        .map_err(Into::into)
    }
}
