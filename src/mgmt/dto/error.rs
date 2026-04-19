use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Conflict(String),
    Unauthorized,
    Internal(String),

    ApplicationStartupFailed { reason: String },
    ServerSecretNotConfigured,
    DatabaseUrlNotConfigured,
    JwtPrivateKeyNotConfigured,
    JwtPublicKeyNotConfigured,
    PageserverUrlNotConfigured,
    PortRangeMisconfigured { value: String },
    CryptoProviderInitFailed { reason: String },
    DatabaseMigrationsFailed { reason: String },
    DatabasePoolInitFailed { reason: String },
    DatabasePoolNotInitialized,
    WorkingDirectoryInvalid { path: String },

    LoginFailed { reason: String },
    RegistrationFailed { reason: String },
    TokenGenerationFailed { reason: String },
    TokenValidationFailed { reason: String },
    AuthKeyGenerationFailed { reason: String },
    AuthKeyUnreadable { path: String, reason: String },

    OrganizationCreationFailed { reason: String },
    OrganizationDeletionFailed { reason: String },
    MemberAdditionFailed { reason: String },
    MemberRemovalFailed { reason: String },
    MemberListingFailed { reason: String },

    ProjectCreationFailed { reason: String },
    ProjectDeletionFailed { reason: String },
    ProjectConfigFetchFailed { reason: String },
    ProjectConfigUpdateFailed { reason: String },

    BranchCreationFailed { reason: String },
    BranchDeletionFailed { reason: String },
    BranchListingFailed { reason: String },
    BranchUpdateFailed { reason: String },
    LsnResolutionFailed { reason: String },
    DurabilityCheckFailed { reason: String },
    TenantIdInvalid { value: String },
    TimelineIdInvalid { value: String },

    ComputeStartupFailed { reason: String },
    ComputeShutdownFailed { reason: String },
    ComputeRecoveryFailed { reason: String },
    ComputeProcessStartupFailed { reason: String },
    ComputePortAllocationFailed,
    ComputeCertificateGenerationFailed { component: String, reason: String },
    ComputeSocketAddressInvalid { addr: String },

    SqlExecutionFailed { reason: String },
    EphemeralQueryFailed { reason: String },

    DaemonStartupFailed { reason: String },
    PostgresInitializationFailed { reason: String },
    PostgresStartupFailed { reason: String },
    PostgresShutdownFailed { reason: String },
    PageserverConfigWriteFailed { reason: String },
    SafekeeperRegistrationFailed { reason: String },
    TracerStartupFailed { reason: String },

    PageserverApiFailed { operation: String, reason: String },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "Not found"),
            AppError::Conflict(message) => write!(f, "{}", message),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Internal(message) => write!(f, "{}", message),

            AppError::ApplicationStartupFailed { reason } => {
                write!(f, "Application startup failed: {}", reason)
            }
            AppError::ServerSecretNotConfigured => {
                write!(f, "Server secret is not configured")
            }
            AppError::DatabaseUrlNotConfigured => {
                write!(f, "Database URL is not configured")
            }
            AppError::JwtPrivateKeyNotConfigured => {
                write!(f, "JWT private key is not configured")
            }
            AppError::JwtPublicKeyNotConfigured => {
                write!(f, "JWT public key is not configured")
            }
            AppError::PageserverUrlNotConfigured => {
                write!(f, "Pageserver URL is not configured")
            }
            AppError::PortRangeMisconfigured { value } => {
                write!(f, "Port range is misconfigured: {}", value)
            }
            AppError::CryptoProviderInitFailed { reason } => {
                write!(f, "Crypto provider initialization failed: {}", reason)
            }
            AppError::DatabaseMigrationsFailed { reason } => {
                write!(f, "Database migrations failed: {}", reason)
            }
            AppError::DatabasePoolInitFailed { reason } => {
                write!(f, "Database pool initialization failed: {}", reason)
            }
            AppError::DatabasePoolNotInitialized => {
                write!(f, "Database pool is not initialized")
            }
            AppError::WorkingDirectoryInvalid { path } => {
                write!(f, "Working directory is invalid: {}", path)
            }

            AppError::LoginFailed { reason } => write!(f, "Login failed: {}", reason),
            AppError::RegistrationFailed { reason } => write!(f, "Registration failed: {}", reason),
            AppError::TokenGenerationFailed { reason } => {
                write!(f, "Token generation failed: {}", reason)
            }
            AppError::TokenValidationFailed { reason } => {
                write!(f, "Token validation failed: {}", reason)
            }
            AppError::AuthKeyGenerationFailed { reason } => {
                write!(f, "Auth key generation failed: {}", reason)
            }
            AppError::AuthKeyUnreadable { path, reason } => {
                write!(f, "Auth key at {} is unreadable: {}", path, reason)
            }

            AppError::OrganizationCreationFailed { reason } => {
                write!(f, "Organization creation failed: {}", reason)
            }
            AppError::OrganizationDeletionFailed { reason } => {
                write!(f, "Organization deletion failed: {}", reason)
            }
            AppError::MemberAdditionFailed { reason } => {
                write!(f, "Member addition failed: {}", reason)
            }
            AppError::MemberRemovalFailed { reason } => {
                write!(f, "Member removal failed: {}", reason)
            }
            AppError::MemberListingFailed { reason } => {
                write!(f, "Member listing failed: {}", reason)
            }

            AppError::ProjectCreationFailed { reason } => {
                write!(f, "Project creation failed: {}", reason)
            }
            AppError::ProjectDeletionFailed { reason } => {
                write!(f, "Project deletion failed: {}", reason)
            }
            AppError::ProjectConfigFetchFailed { reason } => {
                write!(f, "Project config fetch failed: {}", reason)
            }
            AppError::ProjectConfigUpdateFailed { reason } => {
                write!(f, "Project config update failed: {}", reason)
            }

            AppError::BranchCreationFailed { reason } => {
                write!(f, "Branch creation failed: {}", reason)
            }
            AppError::BranchDeletionFailed { reason } => {
                write!(f, "Branch deletion failed: {}", reason)
            }
            AppError::BranchListingFailed { reason } => {
                write!(f, "Branch listing failed: {}", reason)
            }
            AppError::BranchUpdateFailed { reason } => {
                write!(f, "Branch update failed: {}", reason)
            }
            AppError::LsnResolutionFailed { reason } => {
                write!(f, "LSN resolution failed: {}", reason)
            }
            AppError::DurabilityCheckFailed { reason } => {
                write!(f, "Durability check failed: {}", reason)
            }
            AppError::TenantIdInvalid { value } => {
                write!(f, "Tenant ID is invalid: {}", value)
            }
            AppError::TimelineIdInvalid { value } => {
                write!(f, "Timeline ID is invalid: {}", value)
            }

            AppError::ComputeStartupFailed { reason } => {
                write!(f, "Compute startup failed: {}", reason)
            }
            AppError::ComputeShutdownFailed { reason } => {
                write!(f, "Compute shutdown failed: {}", reason)
            }
            AppError::ComputeRecoveryFailed { reason } => {
                write!(f, "Compute recovery failed: {}", reason)
            }
            AppError::ComputeProcessStartupFailed { reason } => {
                write!(f, "Compute process startup failed: {}", reason)
            }
            AppError::ComputePortAllocationFailed => {
                write!(f, "Compute port allocation failed")
            }
            AppError::ComputeCertificateGenerationFailed { component, reason } => {
                write!(
                    f,
                    "Compute certificate generation failed for {}: {}",
                    component, reason
                )
            }
            AppError::ComputeSocketAddressInvalid { addr } => {
                write!(f, "Compute socket address is invalid: {}", addr)
            }

            AppError::SqlExecutionFailed { reason } => {
                write!(f, "SQL execution failed: {}", reason)
            }
            AppError::EphemeralQueryFailed { reason } => {
                write!(f, "Ephemeral query failed: {}", reason)
            }

            AppError::DaemonStartupFailed { reason } => {
                write!(f, "Daemon startup failed: {}", reason)
            }
            AppError::PostgresInitializationFailed { reason } => {
                write!(f, "Postgres initialization failed: {}", reason)
            }
            AppError::PostgresStartupFailed { reason } => {
                write!(f, "Postgres startup failed: {}", reason)
            }
            AppError::PostgresShutdownFailed { reason } => {
                write!(f, "Postgres shutdown failed: {}", reason)
            }
            AppError::PageserverConfigWriteFailed { reason } => {
                write!(f, "Pageserver config write failed: {}", reason)
            }
            AppError::SafekeeperRegistrationFailed { reason } => {
                write!(f, "Safekeeper registration failed: {}", reason)
            }
            AppError::TracerStartupFailed { reason } => {
                write!(f, "Tracer startup failed: {}", reason)
            }

            AppError::PageserverApiFailed { operation, reason } => {
                write!(f, "Pageserver API {} failed: {}", operation, reason)
            }
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized
            | AppError::TokenValidationFailed { .. }
            | AppError::LoginFailed { .. } => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::TenantIdInvalid { .. }
            | AppError::TimelineIdInvalid { .. }
            | AppError::ComputeSocketAddressInvalid { .. }
            | AppError::PortRangeMisconfigured { .. }
            | AppError::RegistrationFailed { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "message": self.to_string() }))).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

impl From<std::net::AddrParseError> for AppError {
    fn from(error: std::net::AddrParseError) -> Self {
        AppError::ComputeSocketAddressInvalid {
            addr: error.to_string(),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Internal(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
