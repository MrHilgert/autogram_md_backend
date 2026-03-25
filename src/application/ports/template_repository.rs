use async_trait::async_trait;
use crate::domain::message_template::MessageTemplate;

#[async_trait]
pub trait TemplateRepository: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<MessageTemplate>, anyhow::Error>;
    async fn set(&self, key: &str, body: &str) -> Result<(), anyhow::Error>;
    async fn list_all(&self) -> Result<Vec<MessageTemplate>, anyhow::Error>;
}
