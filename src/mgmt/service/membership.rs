use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::repository::membership::MembershipRepository;

#[derive(Clone)]
pub struct MembershipService {
    membership_repo: Arc<MembershipRepository>,
}

impl MembershipService {
    pub fn new(membership_repo: Arc<MembershipRepository>) -> Self {
        Self { membership_repo }
    }

    pub async fn verify_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<()> {
        let is_member = self
            .membership_repo
            .exists(user_id, org_id)
            .await?;

        if !is_member {
            return Err(AppError::Unauthorized);
        }

        Ok(())
    }
}
