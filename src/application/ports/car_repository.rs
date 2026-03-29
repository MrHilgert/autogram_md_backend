use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::dto::car::{CreateListingRequest, UpdateListingRequest};
use crate::domain::car::{CarMake, CarModel, ListingPhoto};

/// Raw row from the feed query (joined with makes, models, primary photo).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FeedRow {
    // listing fields
    pub id: Uuid,
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
    pub views_count: i32,
    pub likes_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    // make
    pub make_id: i32,
    pub make_name: String,
    pub make_slug: String,
    // model
    pub model_id: i32,
    pub model_name: String,
    pub model_slug: String,
    // primary photo (nullable)
    pub photo_id: Option<Uuid>,
    pub photo_url: Option<String>,
    pub photo_thumbnail_url: Option<String>,
    pub status: Option<String>,
    pub removal_reason: Option<String>,
    pub promoted_stars: i32,
    pub boosted_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Raw row from the detail query.
#[derive(Debug, sqlx::FromRow)]
pub struct DetailRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub price: i32,
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
    pub features: serde_json::Value,
    pub location: Option<String>,
    pub views_count: i32,
    pub likes_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // make
    pub make_id: i32,
    pub make_name: String,
    pub make_slug: String,
    // model
    pub model_id: i32,
    pub model_name: String,
    pub model_slug: String,
    // seller
    pub seller_username: Option<String>,
    pub seller_first_name: String,
    pub seller_avatar_url: Option<String>,
    pub seller_telegram_id: i64,
    pub removal_reason: Option<String>,
}

/// Lightweight listing attributes for preference updates.
#[derive(Debug, sqlx::FromRow)]
pub struct ListingAttrsRow {
    pub make_id: i32,
    pub model_id: i32,
    pub body: String,
    pub fuel: String,
    pub transmission: String,
    pub drive: Option<String>,
    pub price: i32,
    pub year: i16,
}

/// Filter parameters for the feed query.
pub struct FeedFilter {
    pub cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor_id: Option<Uuid>,
    pub cursor_price: Option<i32>,
    pub cursor_mileage: Option<i32>,
    pub limit: i64,
    pub sort: String,
    pub make_id: Option<i32>,
    pub model_id: Option<i32>,
    pub year_min: Option<i16>,
    pub year_max: Option<i16>,
    pub price_min: Option<i32>,
    pub price_max: Option<i32>,
    pub mileage_min: Option<i32>,
    pub mileage_max: Option<i32>,
    pub fuel_types: Vec<String>,
    pub body_types: Vec<String>,
    pub transmissions: Vec<String>,
    pub drive_types: Vec<String>,
    pub features: Vec<String>,
}

/// Port for car listing persistence and queries.
#[async_trait]
pub trait CarRepository: Send + Sync {
    async fn feed(&self, filter: &FeedFilter) -> Result<Vec<FeedRow>, anyhow::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DetailRow>, anyhow::Error>;
    async fn get_photos(&self, listing_id: Uuid) -> Result<Vec<ListingPhoto>, anyhow::Error>;
    async fn get_makes(&self) -> Result<Vec<CarMake>, anyhow::Error>;
    async fn get_models_by_make(&self, make_id: i32) -> Result<Vec<CarModel>, anyhow::Error>;
    async fn is_liked(&self, user_id: Uuid, listing_id: Uuid) -> Result<bool, anyhow::Error>;
    async fn is_favorited(&self, user_id: Uuid, listing_id: Uuid) -> Result<bool, anyhow::Error>;
    async fn toggle_like(
        &self,
        user_id: Uuid,
        listing_id: Uuid,
    ) -> Result<(bool, i32), anyhow::Error>;
    async fn toggle_favorite(
        &self,
        user_id: Uuid,
        listing_id: Uuid,
    ) -> Result<bool, anyhow::Error>;
    async fn get_liked_ids(
        &self,
        user_id: Uuid,
        listing_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, anyhow::Error>;
    async fn get_favorited_ids(
        &self,
        user_id: Uuid,
        listing_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, anyhow::Error>;
    async fn increment_views(&self, listing_id: Uuid) -> Result<(), anyhow::Error>;

    // CRUD methods
    async fn create_listing(
        &self,
        user_id: Uuid,
        req: &CreateListingRequest,
    ) -> Result<Uuid, anyhow::Error>;
    async fn update_listing(
        &self,
        id: Uuid,
        req: &UpdateListingRequest,
    ) -> Result<(), anyhow::Error>;
    async fn archive_listing(&self, id: Uuid, reason: &str) -> Result<(), anyhow::Error>;
    async fn is_owner(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, anyhow::Error>;

    // Photo management
    async fn add_photo(
        &self,
        listing_id: Uuid,
        url: String,
        thumbnail_url: Option<String>,
        sort_order: i16,
        is_primary: bool,
    ) -> Result<Uuid, anyhow::Error>;
    async fn get_photos_batch(&self, listing_ids: &[Uuid]) -> Result<HashMap<Uuid, Vec<ListingPhoto>>, anyhow::Error>;
    async fn delete_photo(&self, photo_id: Uuid) -> Result<(), anyhow::Error>;
    async fn count_photos(&self, listing_id: Uuid) -> Result<i64, anyhow::Error>;
    async fn get_photo_by_id(
        &self,
        photo_id: Uuid,
    ) -> Result<Option<ListingPhoto>, anyhow::Error>;

    // User profile queries
    async fn get_user_listings(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error>;

    async fn get_user_favorites(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error>;

    async fn get_user_likes(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error>;

    async fn get_user_archived_listings(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error>;

    async fn count_user_listings(&self, user_id: Uuid) -> Result<i64, anyhow::Error>;

    // Boost & promote
    async fn boost_listing(&self, listing_id: Uuid, user_id: Uuid) -> Result<(), anyhow::Error>;
    async fn add_promoted_stars(&self, listing_id: Uuid, user_id: Uuid, stars: i32) -> Result<i32, anyhow::Error>;
    async fn get_promoted(&self, user_id: Option<Uuid>, limit: i64) -> Result<Vec<FeedRow>, anyhow::Error>;
    async fn decay_promoted_stars(&self, amount: i32) -> Result<u64, anyhow::Error>;

    // Extend listing expiration
    async fn extend_listing(&self, listing_id: Uuid, user_id: Uuid) -> Result<(), anyhow::Error>;

    // Expiration
    async fn expire_old_listings(&self) -> Result<i64, anyhow::Error>;

    // Personalized feed support
    async fn feed_by_ids(&self, ids: &[Uuid]) -> Result<Vec<FeedRow>, anyhow::Error>;
    async fn get_listing_attrs(&self, id: Uuid) -> Result<Option<ListingAttrsRow>, anyhow::Error>;
}
