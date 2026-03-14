pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "endpoint_state"))]
    pub struct EndpointState;
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Varchar,
    }
}

diesel::table! {
    memberships (user_id, organization_id) {
        user_id -> Uuid,
        organization_id -> Uuid,
    }
}

diesel::table! {
    projects (id) {
        id -> Uuid,
        organization_id -> Uuid,
    }
}

diesel::table! {
    branches (id) {
        id -> Uuid,
        name -> Varchar,
        parent_branch_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EndpointState;

    endpoints (branch_id) {
        branch_id -> Uuid,
        state -> EndpointState,
        endpoint_port -> Integer,
    }
}

diesel::joinable!(memberships -> users (user_id));
diesel::joinable!(memberships -> organizations (organization_id));
diesel::joinable!(projects -> organizations (organization_id));
diesel::joinable!(endpoints -> branches (branch_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    organizations,
    memberships,
    projects,
    branches,
    endpoints,
);
