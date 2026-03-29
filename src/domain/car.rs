use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Enums ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "fuel_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FuelType {
    Petrol,
    Diesel,
    GasMethane,
    GasPropane,
    PetrolGasMethane,
    PetrolGasPropane,
    Electric,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "body_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    Sedan,
    Hatchback,
    Wagon,
    Suv,
    Coupe,
    Minivan,
    Pickup,
    Convertible,
    Van,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transmission_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Transmission {
    Manual,
    Automatic,
    Cvt,
    Robot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "drive_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DriveType {
    Fwd,
    Rwd,
    Awd,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "steering_side", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SteeringSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "car_condition", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    New,
    Used,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "listing_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ListingStatus {
    Active,
    Sold,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "removal_reason", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RemovalReason {
    Sold,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "currency_code")]
pub enum Currency {
    USD,
    EUR,
    RUP,
}

// --- Entities ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub currency: Currency,
    pub status: ListingStatus,
    pub make_id: i32,
    pub model_id: i32,
    pub year: i16,
    pub fuel: FuelType,
    pub body: BodyType,
    pub transmission: Transmission,
    pub drive: Option<DriveType>,
    pub engine_displacement_cc: Option<i32>,
    pub horsepower: Option<i16>,
    pub mileage_km: i32,
    pub color: Option<String>,
    pub doors_count: Option<i16>,
    pub steering: SteeringSide,
    pub condition: Condition,
    pub features: serde_json::Value,
    pub location: Option<String>,
    pub views_count: i32,
    pub likes_count: i32,
    pub removal_reason: Option<String>,
    pub removed_at: Option<DateTime<Utc>>,
    pub boosted_at: Option<DateTime<Utc>>,
    pub promoted_stars: i32,
    pub previous_price: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ListingPhoto {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub sort_order: i16,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarMake {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CarModel {
    pub id: i32,
    pub make_id: i32,
    pub name: String,
    pub slug: String,
}

/// Allowed feature keys for car listings
pub const ALLOWED_FEATURES: &[&str] = &[
    "abs", "esp", "airbags", "alarm", "keyless_entry", "xenon",
    "climate_control", "ac", "heated_seats", "parking_sensors",
    "rear_camera", "cruise_control", "leather_interior", "sunroof",
    "navigation", "rain_sensor", "light_sensor", "electric_mirrors",
    "electric_windows", "fog_lights", "alloy_wheels", "tinted_windows",
    "roof_rails", "tow_bar",
];
