pub mod sql_types {
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "pg_version"))]
    pub struct PgVersion;
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "compute_endpoint_status"))]
    pub struct ComputeEndpointStatus;
}
diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
diesel::table! {
    memberships (user_id, organization_id) {
        user_id -> Uuid,
        organization_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PgVersion;
    projects (id) {
        id -> Uuid,
        organization_id -> Uuid,
        name -> Varchar,
        pg_version -> PgVersion,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ComputeEndpointStatus;
    branches (id) {
        id -> Uuid,
        name -> Varchar,
        parent_branch_id -> Nullable<Uuid>,
        timeline_id -> Uuid,
        project_id -> Uuid,
        password -> Text,
        slug -> Varchar,
        recent_status -> Nullable<ComputeEndpointStatus>,
        port -> Nullable<Int4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
diesel::joinable!(memberships -> users (user_id));
diesel::joinable!(memberships -> organizations (organization_id));
diesel::joinable!(projects -> organizations (organization_id));
diesel::joinable!(branches -> projects (project_id));
diesel::allow_tables_to_appear_in_same_query!(
    users,
    organizations,
    memberships,
    projects,
    branches,
);
