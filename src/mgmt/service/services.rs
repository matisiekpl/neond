use crate::mgmt::dto::config::Config;
use crate::mgmt::repository::Repositories;
use crate::mgmt::service::branch::BranchService;
use crate::mgmt::service::daemon::DaemonService;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::membership::MembershipService;
use crate::mgmt::service::organization::OrganizationService;
use crate::mgmt::service::project::ProjectService;
use crate::mgmt::service::sql::SqlService;
use crate::mgmt::service::user::UserService;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct Services {
    user: UserService,
    organization: OrganizationService,
    project: Arc<ProjectService>,
    membership: MembershipService,
    branch: Arc<BranchService>,
    endpoint: Arc<EndpointService>,
    sql: SqlService,
    daemon: Arc<DaemonService>,
}

impl Services {
    pub fn new(
        repositories: &Repositories,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        safekeeper_client: Arc<neon_safekeeper_client::mgmt_api::Client>,
        config: Config,
        shutdown_token: CancellationToken,
    ) -> Self {
        let membership = MembershipService::new(Arc::new(repositories.membership().clone()));
        let endpoint = Arc::new(EndpointService::new(
            config.clone(),
            Arc::new(repositories.branch().clone()),
            Arc::new(repositories.project().clone()),
            Arc::new(membership.clone()),
        ));
        let branch = BranchService::new(
            Arc::new(repositories.branch().clone()),
            Arc::new(repositories.project().clone()),
            Arc::new(membership.clone()),
            Arc::clone(&pageserver_client),
            Arc::clone(&safekeeper_client),
            Arc::clone(&endpoint),
            config.clone(),
        );
        let branch = Arc::new(branch);
        let project = ProjectService::new(
            Arc::new(repositories.project().clone()),
            Arc::new(repositories.organization().clone()),
            Arc::new(membership.clone()),
            Arc::clone(&branch),
            Arc::clone(&pageserver_client),
            Arc::clone(&safekeeper_client),
            config.clone(),
        );
        let project = Arc::new(project);
        let daemon = Arc::new(DaemonService::new(
            config.clone(),
            Arc::clone(&pageserver_client),
            Arc::clone(&endpoint),
            Arc::clone(&branch),
            Arc::new(repositories.branch().clone()),
            Arc::new(repositories.project().clone()),
            Arc::new(repositories.organization().clone()),
            shutdown_token,
        ));
        let sql = SqlService::new(
            config.clone(),
            Arc::new(repositories.branch().clone()),
            Arc::new(repositories.project().clone()),
            Arc::new(membership.clone()),
            Arc::clone(&endpoint),
            Arc::clone(&pageserver_client),
            Arc::clone(&safekeeper_client),
        );
        Self {
            user: UserService::new(Arc::new(repositories.user().clone()), Arc::new(repositories.membership().clone()), config.server_secret.clone()),
            organization: OrganizationService::new(
                Arc::new(repositories.organization().clone()),
                Arc::new(repositories.project().clone()),
                Arc::new(repositories.membership().clone()),
                Arc::new(membership.clone()),
                Arc::new(repositories.user().clone()),
                Arc::clone(&project),
            ),
            project,
            branch,
            endpoint,
            sql,
            membership,
            daemon,
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

    pub fn branch(&self) -> &BranchService {
        &self.branch
    }


    pub fn endpoint(&self) -> &Arc<EndpointService> {
        &self.endpoint
    }

    pub fn sql(&self) -> &SqlService {
        &self.sql
    }

    pub fn daemon(&self) -> &Arc<DaemonService> {
        &self.daemon
    }
}
