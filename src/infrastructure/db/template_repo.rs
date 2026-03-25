use async_trait::async_trait;
use sqlx::PgPool;

use crate::application::ports::template_repository::TemplateRepository;
use crate::domain::message_template::MessageTemplate;

pub struct PgTemplateRepository {
    pool: PgPool,
}

impl PgTemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TemplateRepository for PgTemplateRepository {
    async fn get(&self, key: &str) -> Result<Option<MessageTemplate>, anyhow::Error> {
        let row = sqlx::query_as::<_, MessageTemplate>(
            "SELECT key, body, description, updated_at FROM message_templates WHERE key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn set(&self, key: &str, body: &str) -> Result<(), anyhow::Error> {
        sqlx::query(
            "UPDATE message_templates SET body = $2, updated_at = now() WHERE key = $1",
        )
        .bind(key)
        .bind(body)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<MessageTemplate>, anyhow::Error> {
        let rows = sqlx::query_as::<_, MessageTemplate>(
            "SELECT key, body, description, updated_at FROM message_templates ORDER BY key",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
