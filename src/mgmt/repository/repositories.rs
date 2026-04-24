use crate::mgmt::dto::error::Result;
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::db::get_pool;
use crate::mgmt::repository::membership::MembershipRepository;
use crate::mgmt::repository::metric::MetricRepository;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::repository::user::UserRepository;

pub struct Repositories {
    user: UserRepository,
    project: ProjectRepository,
    organization: OrganizationRepository,
    membership: MembershipRepository,
    branch: BranchRepository,
    metric: MetricRepository,
}

impl Repositories {
    pub async fn new() -> Result<Self> {
        let pool = get_pool().await?;
        Ok(Self {
            user: UserRepository::new(pool.clone()),
            project: ProjectRepository::new(pool.clone()),
            organization: OrganizationRepository::new(pool.clone()),
            membership: MembershipRepository::new(pool.clone()),
            branch: BranchRepository::new(pool.clone()),
            metric: MetricRepository::new(pool.clone()),
        })
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

    pub fn metric(&self) -> &MetricRepository {
        &self.metric
    }
}
