use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExecuteSqlRequest {
    pub sql: String,
    pub lsn: Option<String>,
}