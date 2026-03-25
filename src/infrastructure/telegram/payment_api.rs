use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::application::ports::telegram_payment_api::TelegramPaymentApi;

pub struct TelegramPaymentApiClient {
    client: reqwest::Client,
    api_base: String,
}

impl TelegramPaymentApiClient {
    pub fn new(bot_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base: format!("https://api.telegram.org/bot{}", bot_token),
        }
    }
}

#[derive(Serialize)]
struct CreateInvoiceLinkPayload {
    title: String,
    description: String,
    payload: String,
    provider_token: String,
    currency: String,
    prices: Vec<LabeledPrice>,
}

#[derive(Serialize)]
struct LabeledPrice {
    label: String,
    amount: i32,
}

#[derive(Serialize)]
struct AnswerPreCheckoutPayload {
    pre_checkout_query_id: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[async_trait]
impl TelegramPaymentApi for TelegramPaymentApiClient {
    async fn create_invoice_link(
        &self,
        title: &str,
        description: &str,
        payload: &str,
        currency: &str,
        amount: i32,
    ) -> Result<String, anyhow::Error> {
        let body = CreateInvoiceLinkPayload {
            title: title.to_string(),
            description: description.to_string(),
            payload: payload.to_string(),
            provider_token: String::new(),
            currency: currency.to_string(),
            prices: vec![LabeledPrice { label: title.to_string(), amount }],
        };

        let resp: ApiResponse<String> = self.client
            .post(format!("{}/createInvoiceLink", self.api_base))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        match (resp.ok, resp.result) {
            (true, Some(url)) => Ok(url),
            _ => Err(anyhow::anyhow!("createInvoiceLink failed: {}", resp.description.unwrap_or_default())),
        }
    }

    async fn answer_pre_checkout_query(
        &self,
        pre_checkout_query_id: &str,
        ok: bool,
        error_message: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let body = AnswerPreCheckoutPayload {
            pre_checkout_query_id: pre_checkout_query_id.to_string(),
            ok,
            error_message: error_message.map(|s| s.to_string()),
        };

        let resp: ApiResponse<bool> = self.client
            .post(format!("{}/answerPreCheckoutQuery", self.api_base))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            anyhow::bail!("answerPreCheckoutQuery failed: {}", resp.description.unwrap_or_default());
        }
        Ok(())
    }
}
