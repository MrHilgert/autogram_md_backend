use async_trait::async_trait;
use serde::Serialize;

use crate::application::ports::notification_sender::NotificationSender;

pub struct TelegramNotificationSender {
    client: reqwest::Client,
    api_base: String,
}

impl TelegramNotificationSender {
    pub fn new(bot_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base: format!("https://api.telegram.org/bot{}", bot_token),
        }
    }
}

#[async_trait]
impl NotificationSender for TelegramNotificationSender {
    async fn send_html(
        &self,
        telegram_id: i64,
        html: &str,
        button_text: Option<&str>,
        button_url: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let reply_markup = match (button_text, button_url) {
            (Some(text), Some(url)) => Some(InlineKeyboardMarkup {
                inline_keyboard: vec![vec![InlineKeyboardButton {
                    text: text.to_string(),
                    web_app: Some(WebAppInfo {
                        url: url.to_string(),
                    }),
                    url: None,
                }]],
            }),
            _ => None,
        };

        let payload = SendMessagePayload {
            chat_id: telegram_id,
            text: html.to_string(),
            parse_mode: Some("HTML".to_string()),
            reply_markup,
        };

        let resp = self
            .client
            .post(format!("{}/sendMessage", self.api_base))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(
                telegram_id = telegram_id,
                "Telegram sendMessage failed: {}",
                body
            );
            return Err(anyhow::anyhow!("Telegram API error: {}", body));
        }

        Ok(())
    }

    async fn send_with_url_button(
        &self,
        telegram_id: i64,
        html: &str,
        button_text: &str,
        button_url: &str,
    ) -> Result<(), anyhow::Error> {
        let reply_markup = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![InlineKeyboardButton {
                text: button_text.to_string(),
                web_app: None,
                url: Some(button_url.to_string()),
            }]],
        };

        let payload = SendMessagePayload {
            chat_id: telegram_id,
            text: html.to_string(),
            parse_mode: Some("HTML".to_string()),
            reply_markup: Some(reply_markup),
        };

        let resp = self
            .client
            .post(format!("{}/sendMessage", self.api_base))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(telegram_id = telegram_id, "Telegram sendMessage failed: {}", body);
            return Err(anyhow::anyhow!("Telegram API error: {}", body));
        }

        Ok(())
    }
    async fn send_photo_with_url_button(
        &self,
        telegram_id: i64,
        photo_url: &str,
        caption_html: &str,
        button_text: &str,
        button_url: &str,
    ) -> Result<(), anyhow::Error> {
        let reply_markup = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![InlineKeyboardButton {
                text: button_text.to_string(),
                web_app: None,
                url: Some(button_url.to_string()),
            }]],
        };

        let payload = SendPhotoPayload {
            chat_id: telegram_id,
            photo: photo_url.to_string(),
            caption: Some(caption_html.to_string()),
            parse_mode: Some("HTML".to_string()),
            reply_markup: Some(reply_markup),
        };

        let resp = self
            .client
            .post(format!("{}/sendPhoto", self.api_base))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(telegram_id = telegram_id, "Telegram sendPhoto failed: {}", body);
            return Err(anyhow::anyhow!("Telegram API error: {}", body));
        }

        Ok(())
    }
}

// Duplicated from bot.rs (private types there)
#[derive(Debug, Serialize)]
struct SendPhotoPayload {
    chat_id: i64,
    photo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_markup: Option<InlineKeyboardMarkup>,
}

#[derive(Debug, Serialize)]
struct SendMessagePayload {
    chat_id: i64,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_markup: Option<InlineKeyboardMarkup>,
}

#[derive(Debug, Serialize)]
struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Serialize)]
struct InlineKeyboardButton {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    web_app: Option<WebAppInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[derive(Debug, Serialize)]
struct WebAppInfo {
    url: String,
}
