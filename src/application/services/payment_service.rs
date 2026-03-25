use std::sync::Arc;
use uuid::Uuid;
use crate::application::dto::payment::*;
use crate::application::ports::car_repository::CarRepository;
use crate::application::ports::payment_repository::PaymentRepository;
use crate::application::ports::telegram_payment_api::TelegramPaymentApi;
use crate::domain::payment::{Payment, PaymentStatus};

pub struct PaymentService {
    payment_repo: Arc<dyn PaymentRepository>,
    telegram_api: Arc<dyn TelegramPaymentApi>,
    car_repo: Arc<dyn CarRepository>,
}

impl PaymentService {
    pub fn new(
        payment_repo: Arc<dyn PaymentRepository>,
        telegram_api: Arc<dyn TelegramPaymentApi>,
        car_repo: Arc<dyn CarRepository>,
    ) -> Self {
        Self { payment_repo, telegram_api, car_repo }
    }

    pub async fn create_invoice(
        &self,
        user_id: Uuid,
        request: CreateInvoiceRequest,
    ) -> Result<InvoiceResponse, anyhow::Error> {
        if request.amount <= 0 {
            anyhow::bail!("Amount must be positive");
        }

        let payment = self.payment_repo.create(
            user_id, request.amount, "XTR", request.payload,
        ).await?;

        let invoice_url = self.telegram_api.create_invoice_link(
            &request.title,
            &request.description,
            &payment.id.to_string(),
            "XTR",
            request.amount,
        ).await?;

        Ok(InvoiceResponse {
            invoice_url,
            payment_id: payment.id.to_string(),
        })
    }

    pub async fn handle_pre_checkout(
        &self,
        pre_checkout_query_id: &str,
        payload: &str,
        _from_telegram_id: i64,
    ) -> Result<(), anyhow::Error> {
        let payment_id = Uuid::parse_str(payload)
            .map_err(|_| anyhow::anyhow!("Invalid payment payload"))?;

        let payment = self.payment_repo.find_by_id(payment_id).await?;
        let ok = matches!(payment, Some(p) if p.status == PaymentStatus::Pending);

        self.telegram_api.answer_pre_checkout_query(
            pre_checkout_query_id,
            ok,
            if !ok { Some("Payment not found or already processed") } else { None },
        ).await?;

        Ok(())
    }

    pub async fn handle_successful_payment(
        &self,
        payload: &str,
        telegram_payment_charge_id: &str,
        provider_payment_charge_id: &str,
    ) -> Result<Payment, anyhow::Error> {
        let payment_id = Uuid::parse_str(payload)
            .map_err(|_| anyhow::anyhow!("Invalid payment payload"))?;

        let payment = self.payment_repo.confirm(
            payment_id,
            telegram_payment_charge_id,
            provider_payment_charge_id,
        ).await?;

        tracing::info!(
            payment_id = %payment.id,
            user_id = %payment.user_id,
            amount = payment.amount,
            "Payment confirmed"
        );

        // Apply payment effect based on payload
        if let Some(payment_type) = payment.payload.get("type").and_then(|v| v.as_str()) {
            match payment_type {
                "boost" => {
                    if let Some(listing_id_str) = payment.payload.get("listingId").and_then(|v| v.as_str()) {
                        let listing_id = Uuid::parse_str(listing_id_str)?;
                        self.car_repo.boost_listing(listing_id, payment.user_id).await?;
                        tracing::info!(listing_id = %listing_id, "Listing boosted");
                    }
                }
                "promote" => {
                    if let Some(listing_id_str) = payment.payload.get("listingId").and_then(|v| v.as_str()) {
                        let listing_id = Uuid::parse_str(listing_id_str)?;
                        let new_total = self.car_repo.add_promoted_stars(listing_id, payment.user_id, payment.amount).await?;
                        tracing::info!(listing_id = %listing_id, new_total = new_total, "Listing promoted");
                    }
                }
                _ => {
                    tracing::warn!("Unknown payment type: {}", payment_type);
                }
            }
        }

        Ok(payment)
    }
}
