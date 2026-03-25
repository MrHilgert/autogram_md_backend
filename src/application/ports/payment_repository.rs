use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::payment::Payment;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        amount: i32,
        currency: &str,
        payload: serde_json::Value,
    ) -> Result<Payment, anyhow::Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, anyhow::Error>;

    async fn confirm(
        &self,
        id: Uuid,
        telegram_payment_charge_id: &str,
        provider_payment_charge_id: &str,
    ) -> Result<Payment, anyhow::Error>;

    async fn mark_failed(&self, id: Uuid) -> Result<(), anyhow::Error>;
}
