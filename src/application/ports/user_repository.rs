use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::user::{NewUser, User};

/// Port for user persistence operations.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_telegram_id(&self, telegram_id: i64) -> Result<Option<User>, anyhow::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error>;
    async fn upsert(&self, user: &NewUser) -> Result<User, anyhow::Error>;
    async fn update_last_active(&self, id: Uuid) -> Result<(), anyhow::Error>;
}
