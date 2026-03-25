use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::user_preference::UserPreference;

#[async_trait]
pub trait PreferenceRepository: Send + Sync {
    async fn get(&self, user_id: Uuid) -> Result<Option<UserPreference>, anyhow::Error>;
    async fn upsert(&self, pref: &UserPreference) -> Result<(), anyhow::Error>;
    /// Return all active listing IDs scored against user preferences, ordered by score DESC.
    async fn scored_feed(
        &self,
        user_id: Uuid,
        pref: &UserPreference,
    ) -> Result<Vec<Uuid>, anyhow::Error>;
}
