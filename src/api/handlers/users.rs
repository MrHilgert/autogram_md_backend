use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::application::dto::auth::UserProfileResponse;
use crate::application::dto::car::PaginationQuery;
use crate::application::ports::user_repository::UserRepository;
use crate::application::services::car_service::CarService;

#[get("/api/users/me")]
pub async fn get_me(
    user: AuthenticatedUser,
    user_repo: web::Data<Arc<dyn UserRepository>>,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let db_user = user_repo
        .find_by_id(user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Find user error: {:?}", e);
            ApiError::Internal("Failed to load user".to_string())
        })?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let listings_count = car_service
        .count_user_listings(user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Count listings error: {:?}", e);
            ApiError::Internal("Failed to count listings".to_string())
        })?;

    let response = UserProfileResponse {
        id: db_user.id.to_string(),
        telegram_id: db_user.telegram_id,
        username: db_user.username,
        first_name: db_user.first_name,
        last_name: db_user.last_name,
        avatar_url: db_user.avatar_url,
        listings_count,
        created_at: db_user.created_at.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/users/me/listings")]
pub async fn get_my_listings(
    query: web::Query<PaginationQuery>,
    user: AuthenticatedUser,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let response = car_service
        .get_user_listings(user.user_id, Some(user.user_id), q.cursor, q.limit)
        .await
        .map_err(|e| {
            tracing::error!("User listings error: {:?}", e);
            ApiError::Internal("Failed to load user listings".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/users/me/favorites")]
pub async fn get_my_favorites(
    query: web::Query<PaginationQuery>,
    user: AuthenticatedUser,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let response = car_service
        .get_user_favorites(user.user_id, q.cursor, q.limit)
        .await
        .map_err(|e| {
            tracing::error!("User favorites error: {:?}", e);
            ApiError::Internal("Failed to load user favorites".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/users/me/likes")]
pub async fn get_my_likes(
    query: web::Query<PaginationQuery>,
    user: AuthenticatedUser,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let response = car_service
        .get_user_likes(user.user_id, q.cursor, q.limit)
        .await
        .map_err(|e| {
            tracing::error!("User likes error: {:?}", e);
            ApiError::Internal("Failed to load user likes".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/users/me/listings/archived")]
pub async fn get_my_archived_listings(
    query: web::Query<PaginationQuery>,
    user: AuthenticatedUser,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let response = car_service
        .get_user_archived(user.user_id, q.cursor, q.limit)
        .await
        .map_err(|e| {
            tracing::error!("User archived listings error: {:?}", e);
            ApiError::Internal("Failed to load archived listings".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/users/{id}")]
pub async fn get_user_profile(
    path: web::Path<Uuid>,
    user_repo: web::Data<Arc<dyn UserRepository>>,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let user_id = path.into_inner();

    let db_user = user_repo
        .find_by_id(user_id)
        .await
        .map_err(|e| {
            tracing::error!("Find user error: {:?}", e);
            ApiError::Internal("Failed to load user".to_string())
        })?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let listings_count = car_service
        .count_user_listings(user_id)
        .await
        .map_err(|e| {
            tracing::error!("Count listings error: {:?}", e);
            ApiError::Internal("Failed to count listings".to_string())
        })?;

    let response = UserProfileResponse {
        id: db_user.id.to_string(),
        telegram_id: db_user.telegram_id,
        username: db_user.username,
        first_name: db_user.first_name,
        last_name: db_user.last_name,
        avatar_url: db_user.avatar_url,
        listings_count,
        created_at: db_user.created_at.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/users/{id}/listings")]
pub async fn get_user_listings_public(
    path: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    user: Option<AuthenticatedUser>,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let target_user_id = path.into_inner();
    let q = query.into_inner();
    let viewer_id = user.map(|u| u.user_id);

    let response = car_service
        .get_user_listings(target_user_id, viewer_id, q.cursor, q.limit)
        .await
        .map_err(|e| {
            tracing::error!("User listings error: {:?}", e);
            ApiError::Internal("Failed to load user listings".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}
