use std::sync::Arc;
use std::time::Duration;
use chrono::Datelike;
use crate::application::ports::car_repository::CarRepository;
use crate::application::services::notification_service::NotificationService;

pub async fn run_saved_search_checker(_svc: Arc<NotificationService>) {
    tracing::info!("Saved search checker DISABLED");
    // Disabled: notifications by saved search templates are turned off.
    // Code preserved — re-enable by uncommenting the loop below.
    // let mut interval = tokio::time::interval(Duration::from_secs(60));
    // loop {
    //     interval.tick().await;
    //     tracing::info!("Running saved search check...");
    //     match svc.check_saved_searches().await {
    //         Ok(()) => tracing::info!("Saved search check completed"),
    //         Err(e) => tracing::error!("Saved search check failed: {:?}", e),
    //     }
    // }
}

pub async fn run_daily_tasks(
    svc: Arc<NotificationService>,
    car_repo: Arc<dyn CarRepository>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(86400));
    loop {
        interval.tick().await; // First tick returns immediately
        tracing::info!("Running daily tasks");

        // Expire old listings
        match car_repo.expire_old_listings().await {
            Ok(count) => tracing::info!(affected = count, "Expired old listings"),
            Err(e) => tracing::error!("Expire old listings failed: {:?}", e),
        }

        // Decay promoted stars (10 per day)
        match car_repo.decay_promoted_stars(10).await {
            Ok(count) => tracing::info!(affected = count, "Promoted stars daily decay"),
            Err(e) => tracing::error!("Promoted stars decay failed: {:?}", e),
        }

        if chrono::Utc::now().weekday() == chrono::Weekday::Mon {
            if let Err(e) = svc.send_seller_stats().await {
                tracing::error!("Seller stats failed: {:?}", e);
            }
        }
        if let Err(e) = svc.check_expiring_listings().await {
            tracing::error!("Expiry check failed: {:?}", e);
        }
    }
}
