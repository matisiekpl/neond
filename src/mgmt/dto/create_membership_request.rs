use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateMembershipRequest {
    pub user_id: Uuid,
    pub organization_id: Uuid,
}
