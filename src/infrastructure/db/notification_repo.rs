use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::notification_repository::NotificationRepository;
use crate::domain::notification::NotificationType;

pub struct PgNotificationRepository {
    pool: PgPool,
}

impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn create(
        &self,
        user_id: Uuid,
        notif_type: NotificationType,
        listing_id: Option<Uuid>,
        message: &str,
    ) -> Result<Uuid, anyhow::Error> {
        let (id,): (Uuid,) = sqlx::query_as(
            r#"INSERT INTO notifications (user_id, notification_type, listing_id, message_text)
               VALUES ($1, $2, $3, $4)
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(notif_type)
        .bind(listing_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn count_today(&self, user_id: Uuid) -> Result<i64, anyhow::Error> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND sent_at > now() - INTERVAL '1 day'",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn was_sent(
        &self,
        user_id: Uuid,
        notif_type: NotificationType,
        listing_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM notifications WHERE user_id = $1 AND notification_type = $2 AND listing_id = $3)",
        )
        .bind(user_id)
        .bind(notif_type)
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn mark_sent(&self, id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE notifications SET telegram_sent = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
