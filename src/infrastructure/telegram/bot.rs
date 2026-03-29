use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::application::services::payment_service::PaymentService;

const WELCOME_TEXT: &str = "Привет! Здесь покупают и продают авто в Приднестровье.\nВсе объявления рядом — листай, выбирай, пиши продавцу.";

pub async fn run_bot(bot_token: String, webapp_url: String, payment_service: Arc<PaymentService>) {
    let client = reqwest::Client::new();
    let api_base = format!("https://api.telegram.org/bot{}", bot_token);
    let mut offset: i64 = 0;

    tracing::info!("Telegram bot polling started");

    loop {
        match get_updates(&client, &api_base, offset).await {
            Ok(updates) => {
                for update in updates {
                    if let Some(new_offset) = update.update_id.checked_add(1) {
                        offset = new_offset;
                    }

                    // Handle pre_checkout_query
                    if let Some(pcq) = &update.pre_checkout_query {
                        if let Err(e) = payment_service.handle_pre_checkout(
                            &pcq.id, &pcq.invoice_payload, pcq.from.id,
                        ).await {
                            tracing::error!("Failed to handle pre_checkout_query: {:?}", e);
                        }
                        continue;
                    }

                    if let Some(message) = &update.message {
                        // Handle successful_payment
                        if let Some(sp) = &message.successful_payment {
                            if let Err(e) = payment_service.handle_successful_payment(
                                &sp.invoice_payload,
                                &sp.telegram_payment_charge_id,
                                &sp.provider_payment_charge_id,
                            ).await {
                                tracing::error!("Failed to handle successful_payment: {:?}", e);
                            }
                            continue;
                        }

                        if let Some(text) = &message.text {
                            if text.starts_with("/start") {
                                let chat_id = message.chat.id;
                                if let Err(e) = send_welcome(&client, &api_base, chat_id, &webapp_url).await {
                                    tracing::error!("Failed to send welcome: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Bot polling error: {:?}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn get_updates(
    client: &reqwest::Client,
    api_base: &str,
    offset: i64,
) -> Result<Vec<Update>, anyhow::Error> {
    let resp: ApiResponse<Vec<Update>> = client
        .get(format!("{}/getUpdates", api_base))
        .query(&[
            ("offset", offset.to_string()),
            ("timeout", "30".to_string()),
            ("allowed_updates", r#"["message","pre_checkout_query"]"#.to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    Ok(resp.result.unwrap_or_default())
}

async fn send_welcome(
    client: &reqwest::Client,
    api_base: &str,
    chat_id: i64,
    webapp_url: &str,
) -> Result<(), anyhow::Error> {
    let keyboard = InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton {
            text: "Открыть АвтоГрам".to_string(),
            web_app: Some(WebAppInfo {
                url: webapp_url.to_string(),
            }),
            url: None,
        }]],
    };

    let payload = SendMessagePayload {
        chat_id,
        text: WELCOME_TEXT.to_string(),
        parse_mode: None,
        reply_markup: Some(keyboard),
    };

    client
        .post(format!("{}/sendMessage", api_base))
        .json(&payload)
        .send()
        .await?;

    Ok(())
}

// --- Telegram API types ---

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    #[allow(dead_code)]
    ok: bool,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct PreCheckoutQuery {
    id: String,
    from: TgUser,
    #[allow(dead_code)]
    currency: String,
    #[allow(dead_code)]
    total_amount: i32,
    invoice_payload: String,
}

#[derive(Debug, Deserialize)]
struct SuccessfulPayment {
    #[allow(dead_code)]
    currency: String,
    #[allow(dead_code)]
    total_amount: i32,
    invoice_payload: String,
    telegram_payment_charge_id: String,
    provider_payment_charge_id: String,
}

#[derive(Debug, Deserialize)]
struct TgUser {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
    pre_checkout_query: Option<PreCheckoutQuery>,
}

#[derive(Debug, Deserialize)]
struct Message {
    chat: Chat,
    text: Option<String>,
    successful_payment: Option<SuccessfulPayment>,
}

#[derive(Debug, Deserialize)]
struct Chat {
    id: i64,
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
