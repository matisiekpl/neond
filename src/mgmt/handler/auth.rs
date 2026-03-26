use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

pub struct UserId(pub Uuid);

impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let fut = async move {
            let headers = &parts.headers;

            let auth_header = headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .ok_or(AppError::Unauthorized)?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(AppError::Unauthorized)?;

            let server_secret = std::env::var("SERVER_SECRET")
                .map_err(|_| AppError::Internal("SERVER_SECRET not configured".into()))?;

            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(server_secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| AppError::Unauthorized)?;

            Ok(UserId(token_data.claims.sub))
        };
        fut
    }
}
