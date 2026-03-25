use async_trait::async_trait;

#[async_trait]
pub trait TelegramPaymentApi: Send + Sync {
    async fn create_invoice_link(
        &self,
        title: &str,
        description: &str,
        payload: &str,
        currency: &str,
        amount: i32,
    ) -> Result<String, anyhow::Error>;

    async fn answer_pre_checkout_query(
        &self,
        pre_checkout_query_id: &str,
        ok: bool,
        error_message: Option<&str>,
    ) -> Result<(), anyhow::Error>;
}
