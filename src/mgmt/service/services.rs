use std::sync::Arc;

use crate::mgmt::repository::Repositories;
use crate::mgmt::service::membership::MembershipService;
use crate::mgmt::service::organization::OrganizationService;
use crate::mgmt::service::project::ProjectService;
use crate::mgmt::service::user::UserService;

pub struct Services {
    user: UserService,
    organization: OrganizationService,
    project: ProjectService,
    membership: MembershipService,
}

impl Services {
    pub fn new(repositories: &Repositories, jwt_secret: String) -> Self {
        let membership = MembershipService::new(Arc::new(repositories.membership().clone()));

        Self {
            user: UserService::new(Arc::new(repositories.user().clone()), jwt_secret),
            organization: OrganizationService::new(
                Arc::new(repositories.organization().clone()),
                Arc::new(repositories.membership().clone()),
                Arc::new(membership.clone()),
            ),
            project: ProjectService::new(
                Arc::new(repositories.project().clone()),
                Arc::new(repositories.organization().clone()),
                Arc::new(membership.clone()),
            ),
            membership,
        }
    }

    pub fn user(&self) -> &UserService {
        &self.user
    }

    pub fn organization(&self) -> &OrganizationService {
        &self.organization
    }

    pub fn project(&self) -> &ProjectService {
        &self.project
    }

    pub fn membership(&self) -> &MembershipService {
        &self.membership
    }
}
