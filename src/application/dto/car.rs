use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub make_id: Option<i32>,
    pub model_id: Option<i32>,
    pub year_min: Option<i16>,
    pub year_max: Option<i16>,
    pub price_min: Option<i32>,
    pub price_max: Option<i32>,
    pub mileage_min: Option<i32>,
    pub mileage_max: Option<i32>,
    pub fuel: Option<String>,
    pub body: Option<String>,
    pub transmission: Option<String>,
    pub drive: Option<String>,
    pub features: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedResponse {
    pub items: Vec<CarSummaryResponse>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub promoted: Vec<CarSummaryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarSummaryResponse {
    pub id: String,
    pub title: String,
    pub price: i32,
    pub currency: String,
    pub year: i16,
    pub mileage_km: i32,
    pub fuel: String,
    pub transmission: String,
    pub body: String,
    pub drive: Option<String>,
    pub horsepower: Option<i16>,
    pub location: Option<String>,
    pub make: MakeResponse,
    pub model: ModelResponse,
    pub photo: Option<PhotoResponse>,
    pub views_count: i32,
    pub likes_count: i32,
    pub is_liked: bool,
    pub is_favorited: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_stars: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boosted_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarDetailResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<i32>,
    pub currency: String,
    pub status: String,
    pub year: i16,
    pub mileage_km: i32,
    pub fuel: String,
    pub transmission: String,
    pub body: String,
    pub drive: Option<String>,
    pub engine_displacement_cc: Option<i32>,
    pub horsepower: Option<i16>,
    pub color: Option<String>,
    pub doors_count: Option<i16>,
    pub steering: String,
    pub condition: String,
    pub features: Vec<String>,
    pub location: Option<String>,
    pub make: MakeResponse,
    pub model: ModelResponse,
    pub photos: Vec<PhotoResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller: Option<SellerResponse>,
    pub views_count: i32,
    pub likes_count: i32,
    pub is_liked: bool,
    pub is_favorited: bool,
    pub is_owner: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MakeResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct ModelResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoResponse {
    pub id: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub sort_order: i16,
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SellerResponse {
    pub id: String,
    pub username: Option<String>,
    pub first_name: String,
    pub avatar_url: Option<String>,
    pub telegram_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeResponse {
    pub liked: bool,
    pub likes_count: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteResponse {
    pub favorited: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateListingRequest {
    #[serde(default)]
    pub title: String,
    pub description: Option<String>,
    #[validate(range(min = 0))]
    pub price: i32,
    pub currency: Option<String>,
    pub make_id: i32,
    pub model_id: i32,
    #[validate(range(min = 1900, max = 2100))]
    pub year: i16,
    pub fuel: String,
    pub body: String,
    pub transmission: String,
    pub drive: Option<String>,
    pub engine_displacement_cc: Option<i32>,
    pub horsepower: Option<i16>,
    #[validate(range(min = 0))]
    pub mileage_km: i32,
    pub color: Option<String>,
    pub doors_count: Option<i16>,
    pub steering: Option<String>,
    pub condition: Option<String>,
    pub features: Option<Vec<String>>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateListingRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 1))]
    pub price: Option<i32>,
    pub currency: Option<String>,
    #[validate(range(min = 1900, max = 2100))]
    pub year: Option<i16>,
    pub fuel: Option<String>,
    pub body: Option<String>,
    pub transmission: Option<String>,
    pub drive: Option<String>,
    pub engine_displacement_cc: Option<i32>,
    pub horsepower: Option<i16>,
    #[validate(range(min = 0))]
    pub mileage_km: Option<i32>,
    pub color: Option<String>,
    pub doors_count: Option<i16>,
    pub steering: Option<String>,
    pub condition: Option<String>,
    pub features: Option<Vec<String>>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptionsResponse {
    pub fuel_types: Vec<&'static str>,
    pub body_types: Vec<&'static str>,
    pub transmission_types: Vec<&'static str>,
    pub drive_types: Vec<&'static str>,
    pub features: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveListingRequest {
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedPhotoResponse {
    pub id: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub sort_order: i16,
    pub is_primary: bool,
}
