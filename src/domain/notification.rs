use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    PriceDrop,
    NewBySearch,
    SellerStats,
    ListingExpiring,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub listing_id: Option<Uuid>,
    pub message_text: String,
    pub sent_at: DateTime<Utc>,
    pub telegram_sent: bool,
}
