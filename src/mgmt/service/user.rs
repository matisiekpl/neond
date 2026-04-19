use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::login_user_request::LoginUserRequest;
use crate::mgmt::dto::login_user_response::LoginUserResponse;
use crate::mgmt::dto::create_user_request::CreateUserRequest;
use crate::mgmt::dto::register_user_request::RegisterUserRequest;
use crate::mgmt::dto::update_user_request::UpdateUserRequest;
use crate::mgmt::dto::register_user_response::RegisterUserResponse;
use crate::mgmt::dto::setup_response::SetupResponse;
use crate::mgmt::dto::user_response::UserResponse;
use crate::mgmt::repository::membership::MembershipRepository;
use crate::mgmt::repository::user::UserRepository;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

pub struct UserService {
    user_repo: Arc<UserRepository>,
    membership_repo: Arc<MembershipRepository>,
    server_secret: String,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>, membership_repo: Arc<MembershipRepository>, server_secret: String) -> Self {
        Self {
            user_repo,
            membership_repo,
            server_secret,
        }
    }

    pub async fn login(&self, req: LoginUserRequest) -> Result<LoginUserResponse> {
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let hash = user.password_hash.clone();
        let password = req.password.clone();
        let valid = tokio::task::spawn_blocking(move || verify(&password, &hash))
            .await
            .map_err(|error| AppError::LoginFailed {
                reason: error.to_string(),
            })?
            .map_err(|error| AppError::LoginFailed {
                reason: error.to_string(),
            })?;

        if !valid {
            return Err(AppError::Unauthorized);
        }

        let token = self.generate_token(user.id)?;
        Ok(LoginUserResponse { token })
    }

    pub async fn setup(&self) -> Result<SetupResponse> {
        let user_count = self.user_repo.count().await?;
        Ok(SetupResponse {
            registration_open: user_count == 0,
        })
    }

    pub async fn register(&self, req: RegisterUserRequest) -> Result<RegisterUserResponse> {
        let user_count = self.user_repo.count().await?;
        if user_count > 0 {
            return Err(AppError::RegistrationClosed);
        }

        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict("Email already exists".into()));
        }

        let password = req.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
            .await
            .map_err(|error| AppError::RegistrationFailed {
                reason: error.to_string(),
            })?
            .map_err(|error| AppError::RegistrationFailed {
                reason: error.to_string(),
            })?;

        let user = self
            .user_repo
            .create(Uuid::new_v4(), &req.name, &req.email, &password_hash, true)
            .await?;

        let token = self.generate_token(user.id)?;
        Ok(RegisterUserResponse { token })
    }

    pub async fn me(&self, token: &str) -> Result<UserResponse> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.server_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;
        let user = self
            .user_repo
            .find_by_id(token_data.claims.sub)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            is_admin: user.is_admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }
    fn generate_token(&self, user_id: Uuid) -> Result<String> {
        let exp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| AppError::TokenGenerationFailed {
                reason: error.to_string(),
            })?
            .as_secs() as usize
            + 86400; // 24h

        let claims = Claims { sub: user_id, exp };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.server_secret.as_bytes()),
        )
        .map_err(|error| AppError::TokenGenerationFailed {
            reason: error.to_string(),
        })
    }

    async fn verify_admin(&self, caller_id: Uuid) -> Result<()> {
        let caller = self
            .user_repo
            .find_by_id(caller_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        if !caller.is_admin {
            return Err(AppError::Forbidden);
        }
        Ok(())
    }

    pub async fn list_users(&self, caller_id: Uuid) -> Result<Vec<UserResponse>> {
        self.verify_admin(caller_id).await?;

        let users = self.user_repo.find_all().await?;
        Ok(users
            .into_iter()
            .map(|user| UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                is_admin: user.is_admin,
                created_at: user.created_at,
                updated_at: user.updated_at,
            })
            .collect())
    }

    pub async fn create_user(
        &self,
        caller_id: Uuid,
        req: CreateUserRequest,
    ) -> Result<UserResponse> {
        self.verify_admin(caller_id).await?;

        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict("Email already exists".into()));
        }

        let password = req.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
            .await
            .map_err(|error| AppError::RegistrationFailed {
                reason: error.to_string(),
            })?
            .map_err(|error| AppError::RegistrationFailed {
                reason: error.to_string(),
            })?;

        let user = self
            .user_repo
            .create(Uuid::new_v4(), &req.name, &req.email, &password_hash, false)
            .await?;

        Ok(UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            is_admin: user.is_admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    pub async fn update_user(
        &self,
        caller_id: Uuid,
        target_id: Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserResponse> {
        self.verify_admin(caller_id).await?;

        if caller_id == target_id && !req.is_admin {
            return Err(AppError::Conflict(
                "Cannot remove admin role from yourself".into(),
            ));
        }

        let user = self.user_repo.update(target_id, &req.name, &req.email, req.is_admin).await?;

        Ok(UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            is_admin: user.is_admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    pub async fn reset_password(&self, caller_id: Uuid, target_id: Uuid, password: String) -> Result<()> {
        self.verify_admin(caller_id).await?;

        let password_hash = tokio::task::spawn_blocking(move || hash(&password, DEFAULT_COST))
            .await
            .map_err(|error| AppError::RegistrationFailed {
                reason: error.to_string(),
            })?
            .map_err(|error| AppError::RegistrationFailed {
                reason: error.to_string(),
            })?;

        self.user_repo.update_password(target_id, &password_hash).await
    }

    pub async fn delete_user(&self, caller_id: Uuid, target_id: Uuid) -> Result<()> {
        self.verify_admin(caller_id).await?;

        if caller_id == target_id {
            return Err(AppError::Conflict("Cannot delete yourself".into()));
        }

        self.membership_repo.delete_by_user_id(target_id).await?;
        self.user_repo.delete(target_id).await
    }
}
