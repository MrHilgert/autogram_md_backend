use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpResponse};
use futures_util::TryStreamExt;
use uuid::Uuid;
use validator::Validate;

use crate::api::error::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::application::dto::car::{ArchiveListingRequest, CreateListingRequest, FeedQuery, UpdateListingRequest, UploadedPhotoResponse};
use crate::application::services::car_service::CarService;
use crate::application::services::photo_service::PhotoService;

#[get("/api/feed")]
pub async fn get_feed(
    query: web::Query<FeedQuery>,
    car_service: web::Data<Arc<CarService>>,
    user: Option<AuthenticatedUser>,
) -> Result<HttpResponse, ApiError> {
    let user_id = user.map(|u| u.user_id);
    let response = car_service
        .get_feed(&query.into_inner(), user_id)
        .await
        .map_err(|e| {
            tracing::error!("Feed error: {:?}", e);
            ApiError::Internal("Failed to load feed".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/listings/{id}")]
pub async fn get_listing(
    path: web::Path<Uuid>,
    car_service: web::Data<Arc<CarService>>,
    user: Option<AuthenticatedUser>,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();
    let user_id = user.map(|u| u.user_id);

    let response = car_service
        .get_listing(listing_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Listing detail error: {:?}", e);
            ApiError::Internal("Failed to load listing".to_string())
        })?;

    match response {
        Some(detail) => Ok(HttpResponse::Ok().json(detail)),
        None => Err(ApiError::NotFound("Listing not found".to_string())),
    }
}

#[post("/api/listings/{id}/like")]
pub async fn toggle_like(
    path: web::Path<Uuid>,
    car_service: web::Data<Arc<CarService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();
    let response = car_service
        .toggle_like(user.user_id, listing_id)
        .await
        .map_err(|e| {
            tracing::error!("Toggle like error: {:?}", e);
            ApiError::Internal("Failed to toggle like".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[post("/api/users/me/favorites/{id}")]
pub async fn toggle_favorite(
    path: web::Path<Uuid>,
    car_service: web::Data<Arc<CarService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();
    let response = car_service
        .toggle_favorite(user.user_id, listing_id)
        .await
        .map_err(|e| {
            tracing::error!("Toggle favorite error: {:?}", e);
            ApiError::Internal("Failed to toggle favorite".to_string())
        })?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/makes")]
pub async fn get_makes(
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let makes = car_service.get_makes().await.map_err(|e| {
        tracing::error!("Get makes error: {:?}", e);
        ApiError::Internal("Failed to load makes".to_string())
    })?;

    Ok(HttpResponse::Ok().json(makes))
}

#[get("/api/makes/{id}/models")]
pub async fn get_models(
    path: web::Path<i32>,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let make_id = path.into_inner();
    let models = car_service.get_models(make_id).await.map_err(|e| {
        tracing::error!("Get models error: {:?}", e);
        ApiError::Internal("Failed to load models".to_string())
    })?;

    Ok(HttpResponse::Ok().json(models))
}

// --- CRUD Handlers ---

#[post("/api/listings")]
pub async fn create_listing(
    body: web::Json<CreateListingRequest>,
    car_service: web::Data<Arc<CarService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let detail = car_service
        .create_listing(user.user_id, &req)
        .await
        .map_err(|e| {
            tracing::error!("Create listing error: {:?}", e);
            ApiError::Internal("Failed to create listing".to_string())
        })?;

    Ok(HttpResponse::Created().json(detail))
}

#[put("/api/listings/{id}")]
pub async fn update_listing(
    path: web::Path<Uuid>,
    body: web::Json<UpdateListingRequest>,
    car_service: web::Data<Arc<CarService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();
    let req = body.into_inner();

    req.validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    car_service
        .update_listing(listing_id, user.user_id, &req)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("FORBIDDEN") {
                return ApiError::Forbidden("Forbidden".into());
            }
            tracing::error!("Update listing error: {:?}", e);
            ApiError::Internal("Failed to update listing".to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

#[delete("/api/listings/{id}")]
pub async fn archive_listing(
    path: web::Path<Uuid>,
    body: web::Json<ArchiveListingRequest>,
    car_service: web::Data<Arc<CarService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();
    let reason = &body.reason;

    car_service
        .archive_listing(listing_id, user.user_id, reason)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("FORBIDDEN") {
                return ApiError::Forbidden("Forbidden".into());
            }
            tracing::error!("Archive listing error: {:?}", e);
            ApiError::Internal("Failed to archive listing".to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

// --- Photo Handlers ---

fn is_valid_image_magic(data: &[u8]) -> bool {
    data.len() >= 4 && (
        // JPEG
        (data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF) ||
        // PNG
        (data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47) ||
        // WebP
        (data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP")
    )
}

const MAX_PHOTOS_PER_LISTING: i64 = 20;
const MAX_PHOTO_SIZE: usize = 10 * 1024 * 1024; // 10 MB

#[post("/api/listings/{id}/photos")]
pub async fn upload_photos(
    path: web::Path<Uuid>,
    mut payload: actix_multipart::Multipart,
    car_service: web::Data<Arc<CarService>>,
    photo_service: web::Data<Arc<PhotoService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();

    // Verify ownership
    let is_owner = car_service
        .check_ownership(listing_id, user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Ownership check error: {:?}", e);
            ApiError::Internal("Failed to verify ownership".to_string())
        })?;

    if !is_owner {
        return Err(ApiError::Forbidden("Forbidden".into()));
    }

    // Check current photo count
    let current_count = car_service
        .car_repo_ref()
        .count_photos(listing_id)
        .await
        .map_err(|e| {
            tracing::error!("Count photos error: {:?}", e);
            ApiError::Internal("Failed to count photos".to_string())
        })?;

    let mut uploaded = Vec::new();
    let mut photo_index = current_count as i16;

    while let Some(field) = payload
        .try_next()
        .await
        .map_err(|e| ApiError::Validation(format!("Multipart error: {}", e)))?
    {
        if (photo_index as i64) >= MAX_PHOTOS_PER_LISTING {
            break;
        }

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();

        if !content_type.starts_with("image/") {
            continue;
        }

        // Read field bytes
        let mut data = Vec::new();
        let mut stream = field;
        while let Some(chunk) = stream
            .try_next()
            .await
            .map_err(|e| ApiError::Validation(format!("Read error: {}", e)))?
        {
            data.extend_from_slice(&chunk);
            if data.len() > MAX_PHOTO_SIZE {
                return Err(ApiError::Validation(
                    "Photo exceeds maximum size of 10MB".to_string(),
                ));
            }
        }

        if data.is_empty() {
            continue;
        }

        if !is_valid_image_magic(&data) {
            return Err(ApiError::BadRequest("Invalid image format".into()));
        }

        let is_primary = photo_index == 0;

        let (url, thumb_url) = photo_service
            .upload_photo(listing_id, data, &content_type)
            .await
            .map_err(|e| {
                tracing::error!("Photo upload error: {:?}", e);
                ApiError::Internal("Failed to upload photo".to_string())
            })?;

        let photo_id = car_service
            .car_repo_ref()
            .add_photo(
                listing_id,
                url.clone(),
                Some(thumb_url.clone()),
                photo_index,
                is_primary,
            )
            .await
            .map_err(|e| {
                tracing::error!("Add photo DB error: {:?}", e);
                ApiError::Internal("Failed to save photo record".to_string())
            })?;

        uploaded.push(UploadedPhotoResponse {
            id: photo_id.to_string(),
            url,
            thumbnail_url: Some(thumb_url),
            sort_order: photo_index,
            is_primary,
        });

        photo_index += 1;
    }

    Ok(HttpResponse::Created().json(uploaded))
}

#[delete("/api/listings/{listing_id}/photos/{photo_id}")]
pub async fn delete_photo(
    path: web::Path<(Uuid, Uuid)>,
    car_service: web::Data<Arc<CarService>>,
    photo_service: web::Data<Arc<PhotoService>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let (listing_id, photo_id) = path.into_inner();

    // Verify ownership
    let is_owner = car_service
        .check_ownership(listing_id, user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Ownership check error: {:?}", e);
            ApiError::Internal("Failed to verify ownership".to_string())
        })?;

    if !is_owner {
        return Err(ApiError::Forbidden("Forbidden".into()));
    }

    // Get photo to delete from S3
    let photo = car_service
        .car_repo_ref()
        .get_photo_by_id(photo_id)
        .await
        .map_err(|e| {
            tracing::error!("Get photo error: {:?}", e);
            ApiError::Internal("Failed to load photo".to_string())
        })?
        .ok_or_else(|| ApiError::NotFound("Photo not found".to_string()))?;

    // Delete from S3
    photo_service
        .delete_photo(&photo.url, photo.thumbnail_url.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("S3 delete error: {:?}", e);
            ApiError::Internal("Failed to delete photo from storage".to_string())
        })?;

    // Delete from DB
    car_service
        .car_repo_ref()
        .delete_photo(photo_id)
        .await
        .map_err(|e| {
            tracing::error!("DB delete photo error: {:?}", e);
            ApiError::Internal("Failed to delete photo record".to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

#[post("/api/listings/{id}/extend")]
pub async fn extend_listing(
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();

    car_service
        .extend_listing(listing_id, user.user_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") || msg.contains("not owned") {
                return ApiError::Forbidden("Forbidden".into());
            }
            tracing::error!("Extend listing error: {:?}", e);
            ApiError::Internal("Failed to extend listing".to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

#[get("/api/filters/options")]
pub async fn get_filter_options(
    car_service: web::Data<Arc<CarService>>,
) -> Result<HttpResponse, ApiError> {
    let options = car_service.get_filter_options();
    Ok(HttpResponse::Ok().json(options))
}
