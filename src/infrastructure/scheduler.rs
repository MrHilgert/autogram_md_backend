use std::sync::Arc;
use std::time::Duration;
use crate::application::services::notification_service::NotificationService;

pub async fn run_saved_search_checker(svc: Arc<NotificationService>) {
    tracing::info!("Saved search checker started (interval: 60 sec)");
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        tracing::info!("Running saved search check...");
        match svc.check_saved_searches().await {
            Ok(()) => tracing::info!("Saved search check completed"),
            Err(e) => tracing::error!("Saved search check failed: {:?}", e),
        }
    }
}

pub async fn run_daily_tasks(svc: Arc<NotificationService>) {
    loop {
        // Sleep until next run (every 24h, starting from now)
        tokio::time::sleep(Duration::from_secs(86400)).await;
        tracing::info!("Running daily notification tasks");
        if let Err(e) = svc.send_seller_stats().await {
            tracing::error!("Seller stats failed: {:?}", e);
        }
        if let Err(e) = svc.check_expiring_listings().await {
            tracing::error!("Expiry check failed: {:?}", e);
        }
    }
}
