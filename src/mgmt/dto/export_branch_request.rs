use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Plain,
    Custom,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Plain => "sql",
            ExportFormat::Custom => "dump",
        }
    }

    pub fn pg_dump_format(&self) -> &'static str {
        match self {
            ExportFormat::Plain => "plain",
            ExportFormat::Custom => "custom",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Plain => "application/sql",
            ExportFormat::Custom => "application/octet-stream",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportBranchRequest {
    #[serde(default = "default_database")]
    pub database: String,
    pub format: ExportFormat,
    #[serde(default)]
    pub no_owner: bool,
    #[serde(default)]
    pub no_privileges: bool,
}

fn default_database() -> String {
    "postgres".to_string()
}
