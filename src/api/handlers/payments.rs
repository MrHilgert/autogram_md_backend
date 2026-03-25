use std::sync::Arc;
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use crate::api::error::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::application::dto::payment::CreateInvoiceRequest;
use crate::application::services::car_service::CarService;
use crate::application::services::payment_service::PaymentService;

#[derive(Debug, Deserialize)]
pub struct PromoteRequest {
    pub amount: i32,
}

#[post("/api/listings/{id}/boost")]
pub async fn boost_listing(
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
    car_service: web::Data<Arc<CarService>>,
    payment_service: web::Data<Arc<PaymentService>>,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();

    let is_owner = car_service.check_ownership(listing_id, user.user_id).await
        .map_err(|e| { tracing::error!("Ownership check error: {:?}", e); ApiError::Internal("Failed to verify ownership".into()) })?;
    if !is_owner {
        return Err(ApiError::Forbidden("Not the listing owner".into()));
    }

    let invoice = payment_service.create_invoice(
        user.user_id,
        CreateInvoiceRequest {
            title: "Поднять объявление".into(),
            description: "Ваше объявление будет поднято в начало ленты".into(),
            amount: 15,
            payload: serde_json::json!({"type": "boost", "listingId": listing_id.to_string()}),
        },
    ).await.map_err(|e| { tracing::error!("Create boost invoice error: {:?}", e); ApiError::Internal("Failed to create invoice".into()) })?;

    Ok(HttpResponse::Ok().json(invoice))
}

#[post("/api/listings/{id}/promote")]
pub async fn promote_listing(
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
    body: web::Json<PromoteRequest>,
    car_service: web::Data<Arc<CarService>>,
    payment_service: web::Data<Arc<PaymentService>>,
) -> Result<HttpResponse, ApiError> {
    let listing_id = path.into_inner();
    let amount = body.amount;

    if amount < 10 {
        return Err(ApiError::BadRequest("Minimum promotion amount is 10 Stars".into()));
    }

    let is_owner = car_service.check_ownership(listing_id, user.user_id).await
        .map_err(|e| { tracing::error!("Ownership check error: {:?}", e); ApiError::Internal("Failed to verify ownership".into()) })?;
    if !is_owner {
        return Err(ApiError::Forbidden("Not the listing owner".into()));
    }

    let invoice = payment_service.create_invoice(
        user.user_id,
        CreateInvoiceRequest {
            title: format!("Продвижение ({} ⭐)", amount),
            description: "Ваше объявление будет показываться в ленте среди продвинутых".into(),
            amount,
            payload: serde_json::json!({"type": "promote", "listingId": listing_id.to_string()}),
        },
    ).await.map_err(|e| { tracing::error!("Create promote invoice error: {:?}", e); ApiError::Internal("Failed to create invoice".into()) })?;

    Ok(HttpResponse::Ok().json(invoice))
}
