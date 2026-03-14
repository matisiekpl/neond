use std::sync::Arc;

use crate::mgmt::repository::Repositories;
use crate::mgmt::service::user::UserService;

pub struct Services {
    user: UserService,
}

impl Services {
    pub fn new(repositories: &Repositories, jwt_secret: String) -> Self {
        Self {
            user: UserService::new(Arc::new(repositories.user().clone()), jwt_secret),
        }
    }

    pub fn user(&self) -> &UserService {
        &self.user
    }
}
