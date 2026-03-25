use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SavedSearch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub filters: serde_json::Value,
    pub last_checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
