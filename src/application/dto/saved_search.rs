use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSavedSearchRequest {
    pub name: Option<String>,
    pub filters: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSearchResponse {
    pub id: String,
    pub name: String,
    pub filters: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateResponse {
    pub key: String,
    pub body: String,
}
