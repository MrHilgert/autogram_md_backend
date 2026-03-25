use std::sync::Arc;

use actix_web::{get, web, HttpResponse};

use crate::api::error::ApiError;
use crate::application::dto::saved_search::TemplateResponse;
use crate::application::services::template_service::TemplateService;

#[get("/api/templates/{key}")]
pub async fn get_template(
    path: web::Path<String>,
    template_service: web::Data<Arc<TemplateService>>,
) -> Result<HttpResponse, ApiError> {
    let key = path.into_inner();

    let body = template_service.get(&key).await.map_err(|e| {
        tracing::error!("Get template error: {:?}", e);
        ApiError::NotFound(format!("Template '{}' not found", key))
    })?;

    Ok(HttpResponse::Ok().json(TemplateResponse { key, body }))
}
