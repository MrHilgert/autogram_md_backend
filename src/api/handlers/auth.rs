use std::sync::Arc;

use actix_web::{post, web, HttpResponse};

use crate::api::error::ApiError;
use crate::application::dto::auth::{AuthRequest, AuthResponse, UserResponse};
use crate::application::services::auth_service::AuthService;

#[post("/api/auth/telegram")]
pub async fn authenticate(
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<AuthRequest>,
) -> Result<HttpResponse, ApiError> {
    let (token, user) = auth_service
        .authenticate(&body.init_data)
        .await
        .map_err(|e| {
            tracing::warn!("Authentication failed: {:?}", e);
            ApiError::Unauthorized("Authentication failed".to_string())
        })?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}
