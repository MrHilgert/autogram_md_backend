use std::sync::Arc;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod application;
mod config;
mod domain;
mod infrastructure;

use config::AppConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load configuration");
    let bind_addr = format!("{}:{}", config.host, config.port);

    tracing::info!("Starting AutoMarket API server on {}", bind_addr);

    // Create database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Connected to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations applied");

    // Create Redis pool
    let redis_cfg = deadpool_redis::Config::from_url(&config.redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    tracing::info!("Redis pool created");

    // Create application services
    let user_repo: Arc<dyn application::ports::user_repository::UserRepository> =
        Arc::new(infrastructure::db::user_repo::PgUserRepository::new(
            db_pool.clone(),
        ));
    let user_repo_data = web::Data::new(user_repo.clone());
    let auth_service = Arc::new(application::services::auth_service::AuthService::new(
        user_repo,
        config.bot_token.clone(),
        config.jwt_secret.clone(),
    ));
    let auth_data = web::Data::new(auth_service);

    // Create car service
    let car_repo: Arc<dyn application::ports::car_repository::CarRepository> =
        Arc::new(infrastructure::db::car_repo::PgCarRepository::new(
            db_pool.clone(),
        ));
    let view_counter = Arc::new(infrastructure::redis::view_counter::ViewCounter::new(
        redis_pool.clone(),
    ));

    // Preference system (personalized feed)
    let preference_repo: Arc<dyn application::ports::preference_repository::PreferenceRepository> =
        Arc::new(infrastructure::db::preference_repo::PgPreferenceRepository::new(
            db_pool.clone(),
        ));
    let feed_cache = Arc::new(infrastructure::redis::feed_cache::FeedCache::new(
        redis_pool.clone(),
    ));
    let preference_service = Arc::new(
        application::services::preference_service::PreferenceService::new(preference_repo.clone()),
    );

    let car_service = Arc::new(application::services::car_service::CarService::new(
        car_repo.clone(),
        view_counter,
        preference_service,
        feed_cache,
        preference_repo,
    ));
    let car_data = web::Data::new(car_service);

    // Create photo storage and service
    let photo_storage = Arc::new(
        infrastructure::storage::s3::PhotoStorage::new(
            &config.s3_endpoint,
            &config.s3_access_key,
            &config.s3_secret_key,
            &config.s3_region,
            &config.s3_bucket,
            &config.s3_public_url,
        )
        .await,
    );
    let photo_service = Arc::new(
        application::services::photo_service::PhotoService::new(photo_storage),
    );
    let photo_data = web::Data::new(photo_service);

    tracing::info!("Photo storage (S3) initialized");

    // Create payment service
    let telegram_payment_api: Arc<dyn application::ports::telegram_payment_api::TelegramPaymentApi> =
        Arc::new(infrastructure::telegram::payment_api::TelegramPaymentApiClient::new(
            &config.bot_token,
        ));
    let payment_repo: Arc<dyn application::ports::payment_repository::PaymentRepository> =
        Arc::new(infrastructure::db::payment_repo::PgPaymentRepository::new(
            db_pool.clone(),
        ));
    let payment_service = Arc::new(
        application::services::payment_service::PaymentService::new(
            payment_repo,
            telegram_payment_api,
            car_repo.clone(),
        ),
    );
    let payment_data = web::Data::new(payment_service.clone());

    // Template service
    let template_repo: Arc<dyn application::ports::template_repository::TemplateRepository> =
        Arc::new(infrastructure::db::template_repo::PgTemplateRepository::new(
            db_pool.clone(),
        ));
    let template_service = Arc::new(
        application::services::template_service::TemplateService::new(
            template_repo,
            redis_pool.clone(),
        ),
    );

    // Saved search repo
    let saved_search_repo: Arc<dyn application::ports::saved_search_repository::SavedSearchRepository> =
        Arc::new(infrastructure::db::saved_search_repo::PgSavedSearchRepository::new(
            db_pool.clone(),
        ));

    // Notification repo + sender
    let notification_repo: Arc<dyn application::ports::notification_repository::NotificationRepository> =
        Arc::new(infrastructure::db::notification_repo::PgNotificationRepository::new(
            db_pool.clone(),
        ));
    let notification_sender: Arc<dyn application::ports::notification_sender::NotificationSender> =
        Arc::new(
            infrastructure::telegram::notification_sender::TelegramNotificationSender::new(
                &config.bot_token,
            ),
        );

    // Notification service
    let notification_service = Arc::new(
        application::services::notification_service::NotificationService::new(
            notification_repo,
            notification_sender.clone(),
            saved_search_repo.clone(),
            template_service.clone(),
            db_pool.clone(),
            config.webapp_url.clone(),
            "pmrcar_bot".to_string(),
        ),
    );

    // Web data for new services
    let template_data = web::Data::new(template_service);
    let saved_search_data = web::Data::new(saved_search_repo);
    let sender_data = web::Data::new(notification_sender);

    // Start Telegram bot polling in background
    {
        let bot_token = config.bot_token.clone();
        let webapp_url = config.webapp_url.clone();
        let ps = payment_service.clone();
        tokio::spawn(async move {
            infrastructure::telegram::bot::run_bot(bot_token, webapp_url, ps).await;
        });
    }

    // Start notification schedulers
    {
        let ns = notification_service.clone();
        tokio::spawn(async move {
            infrastructure::scheduler::run_saved_search_checker(ns).await;
        });
    }
    {
        let ns = notification_service.clone();
        let cr = car_repo.clone();
        tokio::spawn(async move {
            infrastructure::scheduler::run_daily_tasks(ns, cr).await;
        });
    }

    // Store config for handlers
    let config_data = web::Data::new(config.clone());
    let db_data = web::Data::new(db_pool);
    let redis_data = web::Data::new(redis_pool);

    let governor_conf = GovernorConfigBuilder::default()
        .per_millisecond(200)
        .burst_size(50)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(Governor::new(&governor_conf))
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(config_data.clone())
            .app_data(db_data.clone())
            .app_data(redis_data.clone())
            .app_data(auth_data.clone())
            .app_data(car_data.clone())
            .app_data(photo_data.clone())
            .app_data(payment_data.clone())
            .app_data(user_repo_data.clone())
            .app_data(template_data.clone())
            .app_data(saved_search_data.clone())
            .app_data(sender_data.clone())
            .service(api::handlers::health::health_check)
            .service(api::handlers::auth::authenticate)
            .service(api::handlers::cars::get_feed)
            .service(api::handlers::cars::get_listing)
            .service(api::handlers::cars::toggle_like)
            .service(api::handlers::cars::toggle_favorite)
            .service(api::handlers::cars::get_makes)
            .service(api::handlers::cars::get_models)
            .service(api::handlers::cars::create_listing)
            .service(api::handlers::cars::update_listing)
            .service(api::handlers::cars::archive_listing)
            .service(api::handlers::cars::upload_photos)
            .service(api::handlers::cars::delete_photo)
            .service(api::handlers::cars::contact_seller)
            .service(api::handlers::cars::get_filter_options)
            .service(api::handlers::payments::boost_listing)
            .service(api::handlers::payments::promote_listing)
            .service(api::handlers::users::get_me)
            .service(api::handlers::users::get_my_listings)
            .service(api::handlers::users::get_my_favorites)
            .service(api::handlers::users::get_my_likes)
            .service(api::handlers::users::get_my_archived_listings)
            .service(api::handlers::users::get_user_profile)
            .service(api::handlers::users::get_user_listings_public)
            .service(api::handlers::templates::get_template)
            .service(api::handlers::saved_searches::create_saved_search)
            .service(api::handlers::saved_searches::list_saved_searches)
            .service(api::handlers::saved_searches::delete_saved_search)
            .service(api::handlers::cars::extend_listing)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
