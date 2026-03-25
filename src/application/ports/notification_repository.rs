use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::notification::NotificationType;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, user_id: Uuid, notif_type: NotificationType, listing_id: Option<Uuid>, message: &str) -> Result<Uuid, anyhow::Error>;
    async fn count_today(&self, user_id: Uuid) -> Result<i64, anyhow::Error>;
    async fn was_sent(&self, user_id: Uuid, notif_type: NotificationType, listing_id: Uuid) -> Result<bool, anyhow::Error>;
    async fn mark_sent(&self, id: Uuid) -> Result<(), anyhow::Error>;
}
