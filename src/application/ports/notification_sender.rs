use async_trait::async_trait;

#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_html(&self, telegram_id: i64, html: &str, button_text: Option<&str>, button_url: Option<&str>) -> Result<(), anyhow::Error>;
}
