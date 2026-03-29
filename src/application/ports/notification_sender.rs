use async_trait::async_trait;

#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_html(&self, telegram_id: i64, html: &str, button_text: Option<&str>, button_url: Option<&str>) -> Result<(), anyhow::Error>;
    async fn send_with_url_button(&self, telegram_id: i64, html: &str, button_text: &str, button_url: &str) -> Result<(), anyhow::Error>;
    async fn send_photo_with_url_button(&self, telegram_id: i64, photo_url: &str, caption_html: &str, button_text: &str, button_url: &str) -> Result<(), anyhow::Error>;
}
