use std::future::{ready, Ready};
use std::sync::Arc;

use actix_web::dev::Payload;
use actix_web::{web, FromRequest, HttpRequest};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::application::services::auth_service::AuthService;

/// Extractor that validates the JWT from the `Authorization: Bearer <token>` header
/// and provides the authenticated user's identity to handlers.
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub telegram_id: i64,
}

impl FromRequest for AuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(extract_user(req))
    }
}

fn extract_user(req: &HttpRequest) -> Result<AuthenticatedUser, ApiError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid Authorization format".to_string()))?;

    let auth_service = req
        .app_data::<web::Data<Arc<AuthService>>>()
        .ok_or_else(|| ApiError::Internal("Auth service not configured".to_string()))?;

    let claims = auth_service
        .verify_jwt(token)
        .map_err(|_| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Unauthorized("Invalid token claims".to_string()))?;

    Ok(AuthenticatedUser {
        user_id,
        telegram_id: claims.telegram_id,
    })
}
