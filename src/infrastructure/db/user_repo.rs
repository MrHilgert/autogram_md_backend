use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::user_repository::UserRepository;
use crate::domain::user::{NewUser, User};

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, telegram_id, username, first_name, last_name,
                      avatar_url, created_at, last_active_at
               FROM users
               WHERE telegram_id = $1"#,
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, telegram_id, username, first_name, last_name,
                      avatar_url, created_at, last_active_at
               FROM users
               WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn upsert(&self, new_user: &NewUser) -> Result<User, anyhow::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (telegram_id, username, first_name, last_name, avatar_url)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (telegram_id) DO UPDATE SET
                   username = EXCLUDED.username,
                   first_name = EXCLUDED.first_name,
                   last_name = EXCLUDED.last_name,
                   avatar_url = EXCLUDED.avatar_url,
                   last_active_at = now()
               RETURNING id, telegram_id, username, first_name, last_name,
                         avatar_url, created_at, last_active_at"#,
        )
        .bind(new_user.telegram_id)
        .bind(&new_user.username)
        .bind(&new_user.first_name)
        .bind(&new_user.last_name)
        .bind(&new_user.avatar_url)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update_last_active(&self, id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE users SET last_active_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
