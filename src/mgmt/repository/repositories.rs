use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::db::get_pool;
use crate::mgmt::repository::membership::MembershipRepository;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::repository::user::UserRepository;

pub struct Repositories {
    user: UserRepository,
    project: ProjectRepository,
    organization: OrganizationRepository,
    membership: MembershipRepository,
    branch: BranchRepository,
}

impl Repositories {
    pub async fn new() -> Self {
        let pool = get_pool().await;
        Self {
            user: UserRepository::new(pool.clone()),
            project: ProjectRepository::new(pool.clone()),
            organization: OrganizationRepository::new(pool.clone()),
            membership: MembershipRepository::new(pool.clone()),
            branch: BranchRepository::new(pool.clone()),
        }
    }

    pub fn user(&self) -> &UserRepository {
        &self.user
    }

    pub fn project(&self) -> &ProjectRepository {
        &self.project
    }

    pub fn organization(&self) -> &OrganizationRepository {
        &self.organization
    }

    pub fn membership(&self) -> &MembershipRepository {
        &self.membership
    }

    pub fn branch(&self) -> &BranchRepository {
        &self.branch
    }
}
