use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::saved_search_repository::SavedSearchRepository;
use crate::domain::saved_search::SavedSearch;

pub struct PgSavedSearchRepository {
    pool: PgPool,
}

impl PgSavedSearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SavedSearchRepository for PgSavedSearchRepository {
    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        filters: serde_json::Value,
    ) -> Result<SavedSearch, anyhow::Error> {
        let row = sqlx::query_as::<_, SavedSearch>(
            r#"INSERT INTO saved_searches (user_id, name, filters)
               VALUES ($1, $2, $3)
               RETURNING id, user_id, name, filters, last_checked_at, created_at"#,
        )
        .bind(user_id)
        .bind(name)
        .bind(&filters)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<SavedSearch>, anyhow::Error> {
        let rows = sqlx::query_as::<_, SavedSearch>(
            "SELECT id, user_id, name, filters, last_checked_at, created_at FROM saved_searches WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM saved_searches WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<i64, anyhow::Error> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM saved_searches WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    async fn get_all(&self) -> Result<Vec<SavedSearch>, anyhow::Error> {
        let rows = sqlx::query_as::<_, SavedSearch>(
            "SELECT id, user_id, name, filters, last_checked_at, created_at FROM saved_searches",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn update_last_checked(&self, id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE saved_searches SET last_checked_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
