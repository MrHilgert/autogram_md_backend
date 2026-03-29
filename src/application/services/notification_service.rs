use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::notification_repository::NotificationRepository;
use crate::application::ports::notification_sender::NotificationSender;
use crate::application::ports::saved_search_repository::SavedSearchRepository;
use crate::application::services::template_service::TemplateService;
use crate::domain::notification::NotificationType;

pub struct NotificationService {
    notif_repo: Arc<dyn NotificationRepository>,
    sender: Arc<dyn NotificationSender>,
    search_repo: Arc<dyn SavedSearchRepository>,
    template_service: Arc<TemplateService>,
    db_pool: sqlx::PgPool,
    webapp_url: String,
    bot_username: String,
}

impl NotificationService {
    pub fn new(
        notif_repo: Arc<dyn NotificationRepository>,
        sender: Arc<dyn NotificationSender>,
        search_repo: Arc<dyn SavedSearchRepository>,
        template_service: Arc<TemplateService>,
        db_pool: sqlx::PgPool,
        webapp_url: String,
        bot_username: String,
    ) -> Self {
        Self {
            notif_repo,
            sender,
            search_repo,
            template_service,
            db_pool,
            webapp_url,
            bot_username,
        }
    }

    /// Called when listing price changes
    pub async fn notify_price_drop(
        &self,
        listing_id: Uuid,
        old_price: i32,
        new_price: i32,
        currency: &str,
        title: &str,
    ) -> Result<(), anyhow::Error> {
        if new_price >= old_price {
            return Ok(());
        }

        // Find users who liked or favorited this listing
        let users: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"SELECT DISTINCT u.id, u.telegram_id FROM users u
               WHERE u.id IN (
                   SELECT user_id FROM favorites WHERE listing_id = $1
                   UNION SELECT user_id FROM likes WHERE listing_id = $1
               )"#,
        )
        .bind(listing_id)
        .fetch_all(&self.db_pool)
        .await?;

        let mut params = HashMap::new();
        params.insert("title", title.to_string());
        params.insert("old_price", format!("{} {}", old_price, currency));
        params.insert("new_price", format!("{} {}", new_price, currency));

        let html = self.template_service.render("notif_price_drop", &params).await?;
        let button_url = format!(
            "https://t.me/{}?startapp=car_{}",
            self.bot_username, listing_id
        );

        for (user_id, telegram_id) in users {
            if self
                .notif_repo
                .was_sent(user_id, NotificationType::PriceDrop, listing_id)
                .await
                .unwrap_or(true)
            {
                continue;
            }
            if self.notif_repo.count_today(user_id).await.unwrap_or(10) >= 10 {
                continue;
            }

            let notif_id = self
                .notif_repo
                .create(
                    user_id,
                    NotificationType::PriceDrop,
                    Some(listing_id),
                    &html,
                )
                .await?;
            if self
                .sender
                .send_html(telegram_id, &html, Some("Посмотреть"), Some(&button_url))
                .await
                .is_ok()
            {
                let _ = self.notif_repo.mark_sent(notif_id).await;
            }
        }
        Ok(())
    }

    /// Scheduler: check saved searches for new listings
    pub async fn check_saved_searches(&self) -> Result<(), anyhow::Error> {
        let searches = self.search_repo.get_all().await?;
        for search in searches {
            if let Err(e) = self.process_saved_search(&search).await {
                tracing::error!(search_id = %search.id, "Saved search check failed: {:?}", e);
            }
        }
        Ok(())
    }

    async fn process_saved_search(
        &self,
        search: &crate::domain::saved_search::SavedSearch,
    ) -> Result<(), anyhow::Error> {
        use sqlx::postgres::PgArguments;
        use sqlx::Arguments;

        // Find new active listings matching filters, created after last_checked_at
        let filters = &search.filters;
        let mut conditions = vec!["l.status = 'active'".to_string()];
        let mut param_index: usize = 0;
        let mut args = PgArguments::default();

        // Bind last_checked_at as a parameter
        param_index += 1;
        conditions.push(format!("l.created_at > ${}", param_index));
        args.add(search.last_checked_at)
            .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;

        if let Some(make_id) = filters.get("makeId").and_then(|v| v.as_i64()) {
            param_index += 1;
            conditions.push(format!("l.make_id = ${}", param_index));
            args.add(make_id as i32)
                .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        }
        if let Some(model_id) = filters.get("modelId").and_then(|v| v.as_i64()) {
            param_index += 1;
            conditions.push(format!("l.model_id = ${}", param_index));
            args.add(model_id as i32)
                .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        }
        if let Some(price_min) = filters.get("priceMin").and_then(|v| v.as_i64()) {
            param_index += 1;
            conditions.push(format!("l.price >= ${}", param_index));
            args.add(price_min as i32)
                .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        }
        if let Some(price_max) = filters.get("priceMax").and_then(|v| v.as_i64()) {
            param_index += 1;
            conditions.push(format!("l.price <= ${}", param_index));
            args.add(price_max as i32)
                .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        }
        if let Some(year_min) = filters.get("yearMin").and_then(|v| v.as_i64()) {
            param_index += 1;
            conditions.push(format!("l.year >= ${}", param_index));
            args.add(year_min as i16)
                .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        }
        if let Some(year_max) = filters.get("yearMax").and_then(|v| v.as_i64()) {
            param_index += 1;
            conditions.push(format!("l.year <= ${}", param_index));
            args.add(year_max as i16)
                .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        }

        // Handle fuel filter (stored as array in JSONB) — bind as Vec<String>
        if let Some(fuels) = filters.get("fuel").and_then(|v| v.as_array()) {
            let fuel_list: Vec<String> = fuels.iter()
                .filter_map(|f| f.as_str())
                .map(|s| s.to_string())
                .collect();
            if !fuel_list.is_empty() {
                param_index += 1;
                conditions.push(format!("l.fuel::text = ANY(${})", param_index));
                args.add(fuel_list)
                    .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
            }
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT l.id, l.title, l.price, l.currency::text as currency FROM listings l WHERE {} ORDER BY l.created_at DESC LIMIT 5",
            where_clause
        );

        tracing::info!(search_id = %search.id, search_name = %search.name, "Checking saved search");

        let rows: Vec<(Uuid, String, i32, String)> =
            sqlx::query_as_with(&sql, args).fetch_all(&self.db_pool).await?;

        tracing::info!(search_id = %search.id, found = rows.len(), "Saved search results");

        if rows.is_empty() {
            self.search_repo.update_last_checked(search.id).await?;
            return Ok(());
        }

        // Get user telegram_id
        let user: Option<(i64,)> =
            sqlx::query_as("SELECT telegram_id FROM users WHERE id = $1")
                .bind(search.user_id)
                .fetch_optional(&self.db_pool)
                .await?;
        let telegram_id = match user {
            Some((tid,)) => tid,
            None => return Ok(()),
        };

        if self
            .notif_repo
            .count_today(search.user_id)
            .await
            .unwrap_or(10)
            >= 10
        {
            return Ok(());
        }

        // Build message
        let listings_text: String = rows
            .iter()
            .enumerate()
            .map(|(i, (_, title, price, currency))| {
                format!("{}. <b>{}</b> — {} {}", i + 1, title, price, currency)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut params = HashMap::new();
        params.insert("search_name", search.name.clone());
        params.insert("listings", listings_text);

        let html = self
            .template_service
            .render("notif_new_by_search", &params)
            .await?;
        let first_listing_id = rows[0].0;
        let button_url = format!(
            "https://t.me/{}?startapp=car_{}",
            self.bot_username, first_listing_id
        );

        let notif_id = self
            .notif_repo
            .create(
                search.user_id,
                NotificationType::NewBySearch,
                Some(first_listing_id),
                &html,
            )
            .await?;
        if self
            .sender
            .send_html(telegram_id, &html, Some("Открыть"), Some(&button_url))
            .await
            .is_ok()
        {
            let _ = self.notif_repo.mark_sent(notif_id).await;
        }

        self.search_repo.update_last_checked(search.id).await?;
        Ok(())
    }

    /// Scheduler: send seller stats (weekly, one aggregate message per seller)
    pub async fn send_seller_stats(&self) -> Result<(), anyhow::Error> {
        let rows: Vec<(Uuid, i64, String, i32, i32)> = sqlx::query_as(
            r#"SELECT l.user_id, u.telegram_id, l.title, l.views_count, l.likes_count
               FROM listings l JOIN users u ON u.id = l.user_id
               WHERE l.status = 'active'
               ORDER BY l.user_id, l.created_at DESC"#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Group by seller (user_id, telegram_id)
        let mut sellers: HashMap<(Uuid, i64), Vec<(String, i32, i32)>> = HashMap::new();
        for (user_id, telegram_id, title, views, likes) in rows {
            sellers
                .entry((user_id, telegram_id))
                .or_default()
                .push((title, views, likes));
        }

        for ((user_id, telegram_id), listings) in &sellers {
            let total_views: i32 = listings.iter().map(|(_, v, _)| v).sum();
            let total_likes: i32 = listings.iter().map(|(_, _, l)| l).sum();

            let listing_lines: String = listings
                .iter()
                .map(|(title, views, likes)| {
                    format!("• <b>{}</b> — {} 👁, {} ❤️", title, views, likes)
                })
                .collect::<Vec<_>>()
                .join("\n");

            let html = format!(
                "<b>Статистика за неделю</b>\n\n{}\n\n<b>Итого:</b> {} просмотров, {} лайков",
                listing_lines, total_views, total_likes
            );

            let button_url = format!(
                "https://t.me/{}?startapp=profile_{}",
                self.bot_username, user_id
            );
            let _ = self
                .sender
                .send_html(*telegram_id, &html, Some("Продвинуть"), Some(&button_url))
                .await;
        }
        Ok(())
    }

    /// Scheduler: check expiring listings
    pub async fn check_expiring_listings(&self) -> Result<(), anyhow::Error> {
        let rows: Vec<(Uuid, Uuid, i64, String, i32, String, i64)> = sqlx::query_as(
            r#"SELECT l.id, l.user_id, u.telegram_id, l.title, l.price, l.currency::text,
                      EXTRACT(DAY FROM l.expires_at - now())::bigint as days_left
               FROM listings l JOIN users u ON u.id = l.user_id
               WHERE l.status = 'active' AND l.expires_at IS NOT NULL
               AND l.expires_at BETWEEN now() AND now() + INTERVAL '3 days'"#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        for (listing_id, user_id, telegram_id, title, price, currency, days) in rows {
            if self
                .notif_repo
                .was_sent(user_id, NotificationType::ListingExpiring, listing_id)
                .await
                .unwrap_or(true)
            {
                continue;
            }

            let mut params = HashMap::new();
            params.insert("title", title);
            params.insert("price", format!("{} {}", price, currency));
            params.insert("days", days.to_string());

            if let Ok(html) = self
                .template_service
                .render("notif_listing_expiring", &params)
                .await
            {
                let button_url = format!(
                    "https://t.me/{}?startapp=car_{}",
                    self.bot_username, listing_id
                );
                let notif_id = self
                    .notif_repo
                    .create(
                        user_id,
                        NotificationType::ListingExpiring,
                        Some(listing_id),
                        &html,
                    )
                    .await?;
                if self
                    .sender
                    .send_html(telegram_id, &html, Some("Продлить"), Some(&button_url))
                    .await
                    .is_ok()
                {
                    let _ = self.notif_repo.mark_sent(notif_id).await;
                }
            }
        }
        Ok(())
    }
}
