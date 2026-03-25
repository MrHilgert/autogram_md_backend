use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::application::ports::payment_repository::PaymentRepository;
use crate::domain::payment::{Payment, PaymentStatus};

pub struct PgPaymentRepository {
    pool: PgPool,
}

impl PgPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepository for PgPaymentRepository {
    async fn create(
        &self,
        user_id: Uuid,
        amount: i32,
        currency: &str,
        payload: serde_json::Value,
    ) -> Result<Payment, anyhow::Error> {
        let payment = sqlx::query_as::<_, Payment>(
            r#"INSERT INTO payments (user_id, amount, currency, payload, status)
               VALUES ($1, $2, $3, $4, 'pending')
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(amount)
        .bind(currency)
        .bind(&payload)
        .fetch_one(&self.pool)
        .await?;
        Ok(payment)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, anyhow::Error> {
        let payment = sqlx::query_as::<_, Payment>("SELECT * FROM payments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(payment)
    }

    async fn confirm(
        &self,
        id: Uuid,
        telegram_payment_charge_id: &str,
        provider_payment_charge_id: &str,
    ) -> Result<Payment, anyhow::Error> {
        // Check if already processed (idempotency)
        if let Some(existing) = self.find_by_id(id).await? {
            if existing.status != PaymentStatus::Pending {
                tracing::warn!(payment_id = %id, "Payment already processed, skipping");
                return Ok(existing);
            }
        }

        let payment = sqlx::query_as::<_, Payment>(
            r#"UPDATE payments
               SET status = 'confirmed',
                   telegram_payment_charge_id = $2,
                   provider_payment_charge_id = $3,
                   updated_at = now()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(telegram_payment_charge_id)
        .bind(provider_payment_charge_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(payment)
    }

    async fn mark_failed(&self, id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE payments SET status = 'failed', updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
