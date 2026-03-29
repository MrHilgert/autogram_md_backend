use std::sync::Arc;
use serde::Deserialize;
use crate::application::ports::car_repository::CarRepository;
use crate::application::services::payment_service::PaymentService;
use super::types::{SendMessagePayload, InlineKeyboardMarkup, InlineKeyboardButton, WebAppInfo};

const WELCOME_TEXT: &str = "Привет! Здесь покупают и продают авто в Приднестровье.\nВсе объявления рядом — листай, выбирай, пиши продавцу.";

pub async fn run_bot(
    bot_token: String,
    webapp_url: String,
    payment_service: Arc<PaymentService>,
    car_repo: Arc<dyn CarRepository>,
) {
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

                    // Handle inline_query (share with photo)
                    if let Some(iq) = &update.inline_query {
                        if let Err(e) = handle_inline_query(
                            &client, &api_base, &webapp_url, &car_repo, iq,
                        ).await {
                            tracing::error!("Failed to handle inline_query: {:?}", e);
                        }
                        continue;
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
            ("allowed_updates", r#"["message","pre_checkout_query","inline_query"]"#.to_string()),
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
struct InlineQuery {
    id: String,
    query: String,
}

#[derive(Debug, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
    pre_checkout_query: Option<PreCheckoutQuery>,
    inline_query: Option<InlineQuery>,
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

fn label_fuel(v: &str) -> &str {
    match v {
        "petrol" => "Бензин", "diesel" => "Дизель", "gas_methane" => "Газ (метан)",
        "gas_propane" => "Газ (пропан)", "petrol_gas_methane" => "Бензин/Метан",
        "petrol_gas_propane" => "Бензин/Пропан", "electric" => "Электро", "hybrid" => "Гибрид",
        _ => v,
    }
}

fn label_body(v: &str) -> &str {
    match v {
        "sedan" => "Седан", "hatchback" => "Хэтчбек", "wagon" => "Универсал",
        "suv" => "Внедорожник", "coupe" => "Купе", "minivan" => "Минивэн",
        "pickup" => "Пикап", "convertible" => "Кабриолет", "van" => "Фургон",
        _ => v,
    }
}

fn label_transmission(v: &str) -> &str {
    match v { "manual" => "МКПП", "automatic" => "АКПП", "cvt" => "Вариатор", "robot" => "Робот", _ => v }
}

fn label_drive(v: &str) -> &str {
    match v { "fwd" => "Передний привод", "rwd" => "Задний привод", "awd" => "Полный привод", _ => v }
}

async fn handle_inline_query(
    client: &reqwest::Client,
    api_base: &str,
    webapp_url: &str,
    car_repo: &Arc<dyn CarRepository>,
    iq: &InlineQuery,
) -> Result<(), anyhow::Error> {
    let raw_query = iq.query.trim();
    tracing::info!(query = %raw_query, "Inline query received");

    // Strip optional "car_" prefix from switchInlineQuery
    let query = raw_query.strip_prefix("car_").unwrap_or(raw_query);

    // Parse listing UUID from query
    let listing_id = match uuid::Uuid::parse_str(query) {
        Ok(id) => id,
        Err(_) => {
            tracing::info!("Inline query: invalid UUID, returning empty");
            // Empty or invalid query — return empty results
            answer_inline_query(client, api_base, &iq.id, &serde_json::Value::Array(vec![])).await?;
            return Ok(());
        }
    };

    tracing::info!(listing_id = %listing_id, "Inline query: looking up listing");

    // Fetch listing from DB
    let listing = match car_repo.find_by_id(listing_id).await? {
        Some(l) => {
            tracing::info!(title = %l.title, "Inline query: listing found");
            l
        }
        None => {
            tracing::info!("Inline query: listing not found");
            answer_inline_query(client, api_base, &iq.id, &serde_json::Value::Array(vec![])).await?;
            return Ok(());
        }
    };

    // Get primary photo
    let photos = car_repo.get_photos(listing_id).await?;
    let photo = photos.iter().find(|p| p.is_primary).or(photos.first());

    let app_url = format!("https://t.me/{}?startapp=car_{}", "pmrcar_bot", listing_id);
    let share_url = format!("https://car.hilgert.cc/s/{}", listing_id);

    let formatted_price = listing.price.to_string()
        .as_bytes().rchunks(3).rev()
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>().join(" ");
    let currency_symbol = match listing.currency.as_str() {
        "USD" => "$", "EUR" => "€", "RUP" => "руб.", _ => &listing.currency,
    };
    let price_text = format!("{} {}", formatted_price, currency_symbol);

    let results = if let Some(photo) = photo {
        let thumb = photo.thumbnail_url.as_deref().unwrap_or(&photo.url);
        let photo_url = &photo.url;

        // Build detail lines
        let location = listing.location.as_deref().unwrap_or("—");
        let mileage = if listing.mileage_km > 0 {
            format!("{} км", listing.mileage_km.to_string()
                .as_bytes().rchunks(3).rev()
                .map(|c| std::str::from_utf8(c).unwrap())
                .collect::<Vec<_>>().join(" "))
        } else {
            "0 км".to_string()
        };

        // ⛽ line: engine_cc + fuel + hp (skip missing)
        let mut fuel_parts: Vec<String> = Vec::new();
        if let Some(cc) = listing.engine_displacement_cc {
            fuel_parts.push(format!("{:.1} л", cc as f64 / 1000.0));
        }
        fuel_parts.push(label_fuel(&listing.fuel).to_string());
        if let Some(hp) = listing.horsepower {
            fuel_parts.push(format!("{} л.с.", hp));
        }
        let fuel_line = fuel_parts.join(" · ");

        // 🚗 line: body + transmission + drive (skip missing)
        let mut car_parts: Vec<String> = Vec::new();
        car_parts.push(label_body(&listing.body).to_string());
        car_parts.push(label_transmission(&listing.transmission).to_string());
        if let Some(ref drive) = listing.drive {
            car_parts.push(label_drive(drive).to_string());
        }
        let car_line = car_parts.join(" · ");

        // Hidden link + text — link_preview_options will show photo large above text
        let message_text = format!(
            "<a href=\"{}\">\u{200d}</a><b>{}</b>\n💰 {}\n\n📍 {} · 🛣 {}\n⛽ {}\n🚗 {}",
            share_url, listing.title, price_text,
            location, mileage,
            fuel_line,
            car_line
        );

        serde_json::json!([{
            "type": "article",
            "id": listing_id.to_string(),
            "title": listing.title,
            "description": format!("💰 {}", price_text),
            "thumbnail_url": thumb,
            "input_message_content": {
                "message_text": message_text,
                "parse_mode": "HTML",
                "link_preview_options": {
                    "url": share_url,
                    "prefer_large_media": true,
                    "show_above_text": true
                }
            },
            "reply_markup": {
                "inline_keyboard": [[{
                    "text": "Открыть в АвтоГрам",
                    "url": app_url
                }]]
            }
        }])
    } else {
        let message_text = format!(
            "<b>{}</b>\n💰 {}\n\n<a href=\"{}\">Открыть в АвтоГрам</a>",
            listing.title, price_text, app_url
        );

        serde_json::json!([{
            "type": "article",
            "id": listing_id.to_string(),
            "title": listing.title,
            "description": format!("💰 {}", price_text),
            "input_message_content": {
                "message_text": message_text,
                "parse_mode": "HTML"
            },
            "reply_markup": {
                "inline_keyboard": [[{
                    "text": "Открыть в АвтоГрам",
                    "url": app_url
                }]]
            }
        }])
    };

    tracing::info!("Inline query: sending answer with results");
    answer_inline_query(client, api_base, &iq.id, &results).await
}

async fn answer_inline_query(
    client: &reqwest::Client,
    api_base: &str,
    inline_query_id: &str,
    results: &serde_json::Value,
) -> Result<(), anyhow::Error> {
    let body = serde_json::json!({
        "inline_query_id": inline_query_id,
        "results": results,
        "cache_time": 5,
        "is_personal": false
    });
    let resp = client
        .post(format!("{}/answerInlineQuery", api_base))
        .json(&body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    tracing::info!(status = %status, response = %text, "answerInlineQuery response");
    Ok(())
}

