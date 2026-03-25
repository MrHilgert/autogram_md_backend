use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub telegram_payment_charge_id: Option<String>,
    pub provider_payment_charge_id: Option<String>,
    pub amount: i32,
    pub currency: String,
    pub payload: serde_json::Value,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
