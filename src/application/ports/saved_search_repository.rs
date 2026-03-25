use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::saved_search::SavedSearch;

#[async_trait]
pub trait SavedSearchRepository: Send + Sync {
    async fn create(&self, user_id: Uuid, name: &str, filters: serde_json::Value) -> Result<SavedSearch, anyhow::Error>;
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<SavedSearch>, anyhow::Error>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool, anyhow::Error>;
    async fn count_by_user(&self, user_id: Uuid) -> Result<i64, anyhow::Error>;
    async fn get_all(&self) -> Result<Vec<SavedSearch>, anyhow::Error>;
    async fn update_last_checked(&self, id: Uuid) -> Result<(), anyhow::Error>;
}
