use std::sync::Arc;

use crate::mgmt::repository::Repositories;
use crate::mgmt::service::organization::OrganizationService;
use crate::mgmt::service::user::UserService;

pub struct Services {
    user: UserService,
    organization: OrganizationService,
}

impl Services {
    pub fn new(repositories: &Repositories, jwt_secret: String) -> Self {
        Self {
            user: UserService::new(Arc::new(repositories.user().clone()), jwt_secret),
            organization: OrganizationService::new(
                Arc::new(repositories.organization().clone()),
                Arc::new(repositories.membership().clone()),
            ),
        }
    }

    pub fn user(&self) -> &UserService {
        &self.user
    }

    pub fn organization(&self) -> &OrganizationService {
        &self.organization
    }
}
