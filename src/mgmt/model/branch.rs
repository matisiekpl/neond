use chrono::NaiveDateTime;
use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::branch_response::BranchResponse;
use crate::mgmt::dto::config::Config;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::mgmt::schema::schema::branches)]
pub struct Branch {
    pub id: Uuid,
    pub name: String,
    pub parent_branch_id: Option<Uuid>,
    pub timeline_id: Uuid,
    pub project_id: Uuid,
    pub password: String,
    pub slug: String,
    pub recent_status: Option<ComputeEndpointStatus>,
    pub port: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Branch {
    pub fn get_connection_string(&self, config: Config, port: u16) -> String {
        match config.hostname {
            Some(hostname) => {
                format!(
                    "postgresql://postgres:{}@{}.{}:{}/postgres?sslmode=require&channel_binding=require",
                    self.password, self.slug, hostname, config.pg_proxy_port
                )
            }
            None => {
                format!(
                    "postgresql://postgres:{}@0.0.0.0:{}/postgres?sslmode=require&channel_binding=require",
                    self.password, port,
                )
            }
        }
    }

    pub fn get_pooler_connection_string(&self, config: Config, pooler_port: u16) -> String {
        match config.hostname {
            Some(hostname) => {
                format!(
                    "postgresql://postgres:{}@{}-pooler.{}:{}/postgres?sslmode=require",
                    self.password, self.slug, hostname, config.pg_proxy_port
                )
            }
            None => {
                format!(
                    "postgresql://postgres:{}@0.0.0.0:{}/postgres?sslmode=require",
                    self.password, pooler_port,
                )
            }
        }
    }
}
