use std::collections::HashMap;
use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::application::dto::saved_search::{CreateSavedSearchRequest, SavedSearchResponse};
use crate::application::ports::notification_sender::NotificationSender;
use crate::application::ports::saved_search_repository::SavedSearchRepository;
use crate::application::ports::user_repository::UserRepository;
use crate::application::services::template_service::TemplateService;

const MAX_SAVED_SEARCHES: i64 = 5;

#[post("/api/saved-searches")]
pub async fn create_saved_search(
    body: web::Json<CreateSavedSearchRequest>,
    repo: web::Data<Arc<dyn SavedSearchRepository>>,
    user: AuthenticatedUser,
    user_repo: web::Data<Arc<dyn UserRepository>>,
    sender: web::Data<Arc<dyn NotificationSender>>,
    template_svc: web::Data<Arc<TemplateService>>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    // Validate filters is an object
    if !req.filters.is_object() {
        return Err(ApiError::Validation("filters must be a JSON object".to_string()));
    }

    // Check limit
    let count = repo.count_by_user(user.user_id).await.map_err(|e| {
        tracing::error!("Count saved searches error: {:?}", e);
        ApiError::Internal("Failed to count saved searches".to_string())
    })?;

    if count >= MAX_SAVED_SEARCHES {
        return Err(ApiError::BadRequest(format!(
            "Maximum {} saved searches allowed",
            MAX_SAVED_SEARCHES
        )));
    }

    let name = req.name.unwrap_or_default();
    let search = repo
        .create(user.user_id, &name, req.filters)
        .await
        .map_err(|e| {
            tracing::error!("Create saved search error: {:?}", e);
            ApiError::Internal("Failed to create saved search".to_string())
        })?;

    // Send confirmation via bot (disabled)
    // tokio::spawn({
    //     let sender = sender.get_ref().clone();
    //     let template_svc = template_svc.get_ref().clone();
    //     let user_repo = user_repo.get_ref().clone();
    //     let user_id = user.user_id;
    //     let search_name = search.name.clone();
    //     async move {
    //         if let Ok(Some(db_user)) = user_repo.find_by_id(user_id).await {
    //             let mut params = HashMap::new();
    //             params.insert("name", search_name);
    //             if let Ok(html) = template_svc.render("saved_search_created", &params).await {
    //                 let _ = sender.send_html(db_user.telegram_id, &html, None, None).await;
    //             }
    //         }
    //     }
    // });

    Ok(HttpResponse::Created().json(SavedSearchResponse {
        id: search.id.to_string(),
        name: search.name,
        filters: search.filters,
        created_at: search.created_at.to_rfc3339(),
    }))
}

#[get("/api/saved-searches")]
pub async fn list_saved_searches(
    repo: web::Data<Arc<dyn SavedSearchRepository>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let searches = repo.list_by_user(user.user_id).await.map_err(|e| {
        tracing::error!("List saved searches error: {:?}", e);
        ApiError::Internal("Failed to list saved searches".to_string())
    })?;

    let items: Vec<SavedSearchResponse> = searches
        .into_iter()
        .map(|s| SavedSearchResponse {
            id: s.id.to_string(),
            name: s.name,
            filters: s.filters,
            created_at: s.created_at.to_rfc3339(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({ "data": items })))
}

#[delete("/api/saved-searches/{id}")]
pub async fn delete_saved_search(
    path: web::Path<Uuid>,
    repo: web::Data<Arc<dyn SavedSearchRepository>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    let deleted = repo.delete(id, user.user_id).await.map_err(|e| {
        tracing::error!("Delete saved search error: {:?}", e);
        ApiError::Internal("Failed to delete saved search".to_string())
    })?;

    if !deleted {
        return Err(ApiError::NotFound("Saved search not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
