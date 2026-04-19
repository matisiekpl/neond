use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::login_user_request::LoginUserRequest;
use crate::mgmt::dto::login_user_response::LoginUserResponse;
use crate::mgmt::dto::register_user_request::RegisterUserRequest;
use crate::mgmt::dto::register_user_response::RegisterUserResponse;
use crate::mgmt::dto::user_response::UserResponse;
use crate::mgmt::repository::user::UserRepository;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

pub struct UserService {
    user_repo: Arc<UserRepository>,
    server_secret: String,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>, server_secret: String) -> Self {
        Self {
            user_repo,
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

    pub async fn register(&self, req: RegisterUserRequest) -> Result<RegisterUserResponse> {
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
            .create(Uuid::new_v4(), &req.name, &req.email, &password_hash)
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
}
