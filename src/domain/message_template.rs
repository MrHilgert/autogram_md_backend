use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageTemplate {
    pub key: String,
    pub body: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}
