use serde::Serialize;

#[derive(Serialize)]
pub struct ExecuteSqlResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    pub rows_affected: Option<u64>,
}