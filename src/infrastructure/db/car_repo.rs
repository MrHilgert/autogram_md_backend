use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgArguments;
use sqlx::{Arguments, PgPool};
use uuid::Uuid;

use crate::application::dto::car::{CreateListingRequest, UpdateListingRequest};
use crate::application::ports::car_repository::{
    CarRepository, DetailRow, FeedFilter, FeedRow, ListingAttrsRow,
};
use crate::domain::car::{CarMake, CarModel, ListingPhoto};

pub struct PgCarRepository {
    pool: PgPool,
}

impl PgCarRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Helper to track parameter index for dynamic SQL building.
struct ParamBuilder {
    index: usize,
    args: PgArguments,
}

impl ParamBuilder {
    fn new() -> Self {
        Self {
            index: 0,
            args: PgArguments::default(),
        }
    }

    fn next(&mut self) -> String {
        self.index += 1;
        format!("${}", self.index)
    }

    /// Add a value and return the placeholder string.
    fn add<'q, T>(&mut self, value: T) -> Result<String, anyhow::Error>
    where
        T: sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + 'q,
    {
        let placeholder = self.next();
        self.args
            .add(value)
            .map_err(|e| anyhow::anyhow!("bind error: {}", e))?;
        Ok(placeholder)
    }

    fn into_args(self) -> PgArguments {
        self.args
    }
}

#[async_trait]
impl CarRepository for PgCarRepository {
    async fn feed(&self, filter: &FeedFilter) -> Result<Vec<FeedRow>, anyhow::Error> {
        let mut pb = ParamBuilder::new();
        let mut conditions = vec!["l.status = 'active'".to_string()];

        // Filters
        if let Some(make_id) = filter.make_id {
            let p = pb.add(make_id)?;
            conditions.push(format!("l.make_id = {p}"));
        }
        if let Some(model_id) = filter.model_id {
            let p = pb.add(model_id)?;
            conditions.push(format!("l.model_id = {p}"));
        }
        if let Some(year_min) = filter.year_min {
            let p = pb.add(year_min)?;
            conditions.push(format!("l.year >= {p}"));
        }
        if let Some(year_max) = filter.year_max {
            let p = pb.add(year_max)?;
            conditions.push(format!("l.year <= {p}"));
        }
        if let Some(price_min) = filter.price_min {
            let p = pb.add(price_min)?;
            conditions.push(format!("l.price >= {p}"));
        }
        if let Some(price_max) = filter.price_max {
            let p = pb.add(price_max)?;
            conditions.push(format!("l.price <= {p}"));
        }
        if let Some(mileage_min) = filter.mileage_min {
            let p = pb.add(mileage_min)?;
            conditions.push(format!("l.mileage_km >= {p}"));
        }
        if let Some(mileage_max) = filter.mileage_max {
            let p = pb.add(mileage_max)?;
            conditions.push(format!("l.mileage_km <= {p}"));
        }

        // Array-based filters (fuel, body, transmission, drive)
        if !filter.fuel_types.is_empty() {
            let p = pb.add(filter.fuel_types.clone())?;
            conditions.push(format!("l.fuel::text = ANY({p})"));
        }
        if !filter.body_types.is_empty() {
            let p = pb.add(filter.body_types.clone())?;
            conditions.push(format!("l.body::text = ANY({p})"));
        }
        if !filter.transmissions.is_empty() {
            let p = pb.add(filter.transmissions.clone())?;
            conditions.push(format!("l.transmission::text = ANY({p})"));
        }
        if !filter.drive_types.is_empty() {
            let p = pb.add(filter.drive_types.clone())?;
            conditions.push(format!("l.drive::text = ANY({p})"));
        }

        // Features filter: features @> '["abs","esp"]'::jsonb
        if !filter.features.is_empty() {
            let features_json = serde_json::to_string(&filter.features)?;
            let p = pb.add(features_json)?;
            conditions.push(format!("l.features @> {p}::jsonb"));
        }

        // Cursor + ordering
        let (order_clause, cursor_condition) = match filter.sort.as_str() {
            "price_asc" => {
                let cursor = if let (Some(price), Some(id)) =
                    (filter.cursor_price, filter.cursor_id)
                {
                    let p1 = pb.add(price)?;
                    let p2 = pb.add(id)?;
                    Some(format!("(l.price > {p1} OR (l.price = {p1} AND l.id > {p2}))"))
                } else {
                    None
                };
                ("ORDER BY l.price ASC, l.id ASC".to_string(), cursor)
            }
            "price_desc" => {
                let cursor = if let (Some(price), Some(id)) =
                    (filter.cursor_price, filter.cursor_id)
                {
                    let p1 = pb.add(price)?;
                    let p2 = pb.add(id)?;
                    Some(format!("(l.price < {p1} OR (l.price = {p1} AND l.id < {p2}))"))
                } else {
                    None
                };
                ("ORDER BY l.price DESC, l.id DESC".to_string(), cursor)
            }
            "mileage_asc" => {
                let cursor = if let (Some(mileage), Some(id)) =
                    (filter.cursor_mileage, filter.cursor_id)
                {
                    let p1 = pb.add(mileage)?;
                    let p2 = pb.add(id)?;
                    Some(format!(
                        "(l.mileage_km > {p1} OR (l.mileage_km = {p1} AND l.id > {p2}))"
                    ))
                } else {
                    None
                };
                ("ORDER BY l.mileage_km ASC, l.id ASC".to_string(), cursor)
            }
            // "newest" is default
            _ => {
                let cursor = if let (Some(created_at), Some(id)) =
                    (filter.cursor_created_at, filter.cursor_id)
                {
                    let p1 = pb.add(created_at)?;
                    let p2 = pb.add(id)?;
                    Some(format!(
                        "(COALESCE(l.boosted_at, l.created_at) < {p1} OR (COALESCE(l.boosted_at, l.created_at) = {p1} AND l.id < {p2}))"
                    ))
                } else {
                    None
                };
                ("ORDER BY COALESCE(l.boosted_at, l.created_at) DESC, l.id DESC".to_string(), cursor)
            }
        };

        if let Some(cond) = cursor_condition {
            conditions.push(cond);
        }

        let where_clause = conditions.join(" AND ");

        // Fetch limit + 1 to determine hasMore
        let limit_val = filter.limit + 1;
        let p_limit = pb.add(limit_val)?;

        let sql = format!(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                NULL::text AS status, NULL::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM listings l
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE {where_clause}
            {order_clause}
            LIMIT {p_limit}"#,
        );

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, pb.into_args())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<DetailRow>, anyhow::Error> {
        let row = sqlx::query_as::<_, DetailRow>(
            r#"SELECT
                l.id, l.user_id, l.title, l.description,
                l.price, l.currency::text AS currency, l.status::text AS status,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.engine_displacement_cc, l.horsepower,
                l.color, l.doors_count,
                l.steering::text AS steering, l.condition::text AS condition,
                l.features, l.location,
                l.views_count, l.likes_count,
                l.created_at, l.updated_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                u.username AS seller_username, u.first_name AS seller_first_name,
                u.avatar_url AS seller_avatar_url, u.telegram_id AS seller_telegram_id,
                l.removal_reason::text AS removal_reason
            FROM listings l
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_photos(&self, listing_id: Uuid) -> Result<Vec<ListingPhoto>, anyhow::Error> {
        let photos = sqlx::query_as::<_, ListingPhoto>(
            r#"SELECT id, listing_id, url, thumbnail_url, sort_order, is_primary, created_at
               FROM listing_photos
               WHERE listing_id = $1
               ORDER BY sort_order ASC"#,
        )
        .bind(listing_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(photos)
    }

    async fn get_photos_batch(&self, listing_ids: &[Uuid]) -> Result<HashMap<Uuid, Vec<ListingPhoto>>, anyhow::Error> {
        let photos = sqlx::query_as::<_, ListingPhoto>(
            r#"SELECT id, listing_id, url, thumbnail_url, sort_order, is_primary, created_at
               FROM listing_photos
               WHERE listing_id = ANY($1)
               ORDER BY listing_id, sort_order ASC"#,
        )
        .bind(listing_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut map: HashMap<Uuid, Vec<ListingPhoto>> = HashMap::new();
        for photo in photos {
            map.entry(photo.listing_id).or_default().push(photo);
        }
        Ok(map)
    }

    async fn get_makes(&self) -> Result<Vec<CarMake>, anyhow::Error> {
        let makes = sqlx::query_as::<_, CarMake>(
            "SELECT id, name, slug FROM car_makes ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(makes)
    }

    async fn get_models_by_make(&self, make_id: i32) -> Result<Vec<CarModel>, anyhow::Error> {
        let models = sqlx::query_as::<_, CarModel>(
            "SELECT id, make_id, name, slug FROM car_models WHERE make_id = $1 ORDER BY name ASC",
        )
        .bind(make_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    async fn is_liked(&self, user_id: Uuid, listing_id: Uuid) -> Result<bool, anyhow::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND listing_id = $2)",
        )
        .bind(user_id)
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn is_favorited(&self, user_id: Uuid, listing_id: Uuid) -> Result<bool, anyhow::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE user_id = $1 AND listing_id = $2)",
        )
        .bind(user_id)
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn toggle_like(
        &self,
        user_id: Uuid,
        listing_id: Uuid,
    ) -> Result<(bool, i32), anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        // Check if like exists
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND listing_id = $2)",
        )
        .bind(user_id)
        .bind(listing_id)
        .fetch_one(&mut *tx)
        .await?;

        if exists {
            // Remove like
            sqlx::query("DELETE FROM likes WHERE user_id = $1 AND listing_id = $2")
                .bind(user_id)
                .bind(listing_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                "UPDATE listings SET likes_count = GREATEST(likes_count - 1, 0) WHERE id = $1",
            )
            .bind(listing_id)
            .execute(&mut *tx)
            .await?;
        } else {
            // Add like
            sqlx::query(
                "INSERT INTO likes (user_id, listing_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(listing_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query("UPDATE listings SET likes_count = likes_count + 1 WHERE id = $1")
                .bind(listing_id)
                .execute(&mut *tx)
                .await?;
        }

        // Get new count
        let new_count: i32 =
            sqlx::query_scalar("SELECT likes_count FROM listings WHERE id = $1")
                .bind(listing_id)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;

        Ok((!exists, new_count))
    }

    async fn toggle_favorite(
        &self,
        user_id: Uuid,
        listing_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE user_id = $1 AND listing_id = $2)",
        )
        .bind(user_id)
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        if exists {
            sqlx::query("DELETE FROM favorites WHERE user_id = $1 AND listing_id = $2")
                .bind(user_id)
                .bind(listing_id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("INSERT INTO favorites (user_id, listing_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(listing_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(!exists)
    }

    async fn get_liked_ids(
        &self,
        user_id: Uuid,
        listing_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, anyhow::Error> {
        if listing_ids.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT listing_id FROM likes WHERE user_id = $1 AND listing_id = ANY($2)",
        )
        .bind(user_id)
        .bind(listing_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }

    async fn get_favorited_ids(
        &self,
        user_id: Uuid,
        listing_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, anyhow::Error> {
        if listing_ids.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT listing_id FROM favorites WHERE user_id = $1 AND listing_id = ANY($2)",
        )
        .bind(user_id)
        .bind(listing_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }

    async fn increment_views(&self, listing_id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE listings SET views_count = views_count + 1 WHERE id = $1")
            .bind(listing_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_listing(
        &self,
        user_id: Uuid,
        req: &CreateListingRequest,
    ) -> Result<Uuid, anyhow::Error> {
        let currency = req.currency.as_deref().unwrap_or("USD");
        let steering = req.steering.as_deref().unwrap_or("left");
        let condition = req.condition.as_deref().unwrap_or("used");
        let features_json = match &req.features {
            Some(f) => serde_json::to_value(f)?,
            None => serde_json::Value::Array(vec![]),
        };

        let id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO listings (
                user_id, title, description, price, currency,
                make_id, model_id, year, fuel, body, transmission,
                drive, engine_displacement_cc, horsepower, mileage_km,
                color, doors_count, steering, condition, features, location,
                status, views_count, likes_count
            ) VALUES (
                $1, $2, $3, $4, $5::currency_code,
                $6, $7, $8, $9::fuel_type, $10::body_type, $11::transmission_type,
                $12::drive_type, $13, $14, $15,
                $16, $17, $18::steering_side, $19::car_condition, $20::jsonb, $21,
                'active'::listing_status, 0, 0
            ) RETURNING id"#,
        )
        .bind(user_id)
        .bind(&req.title)
        .bind(req.description.as_deref().unwrap_or(""))
        .bind(req.price)
        .bind(currency)
        .bind(req.make_id)
        .bind(req.model_id)
        .bind(req.year)
        .bind(&req.fuel)
        .bind(&req.body)
        .bind(&req.transmission)
        .bind(req.drive.as_deref())
        .bind(req.engine_displacement_cc)
        .bind(req.horsepower)
        .bind(req.mileage_km)
        .bind(req.color.as_deref())
        .bind(req.doors_count)
        .bind(steering)
        .bind(condition)
        .bind(features_json)
        .bind(req.location.as_deref())
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update_listing(
        &self,
        id: Uuid,
        req: &UpdateListingRequest,
    ) -> Result<(), anyhow::Error> {
        // Build dynamic SET clause
        let mut pb = ParamBuilder::new();
        let mut sets = Vec::new();

        if let Some(ref title) = req.title {
            let p = pb.add(title.clone())?;
            sets.push(format!("title = {p}"));
        }
        if let Some(ref description) = req.description {
            let p = pb.add(description.clone())?;
            sets.push(format!("description = {p}"));
        }
        if let Some(price) = req.price {
            let p = pb.add(price)?;
            sets.push(format!("price = {p}"));
        }
        if let Some(ref currency) = req.currency {
            let p = pb.add(currency.clone())?;
            sets.push(format!("currency = {p}::currency_code"));
        }
        if let Some(year) = req.year {
            let p = pb.add(year)?;
            sets.push(format!("year = {p}"));
        }
        if let Some(ref fuel) = req.fuel {
            let p = pb.add(fuel.clone())?;
            sets.push(format!("fuel = {p}::fuel_type"));
        }
        if let Some(ref body) = req.body {
            let p = pb.add(body.clone())?;
            sets.push(format!("body = {p}::body_type"));
        }
        if let Some(ref transmission) = req.transmission {
            let p = pb.add(transmission.clone())?;
            sets.push(format!("transmission = {p}::transmission_type"));
        }
        if let Some(ref drive) = req.drive {
            let p = pb.add(drive.clone())?;
            sets.push(format!("drive = {p}::drive_type"));
        }
        if let Some(engine_cc) = req.engine_displacement_cc {
            let p = pb.add(engine_cc)?;
            sets.push(format!("engine_displacement_cc = {p}"));
        }
        if let Some(hp) = req.horsepower {
            let p = pb.add(hp)?;
            sets.push(format!("horsepower = {p}"));
        }
        if let Some(mileage) = req.mileage_km {
            let p = pb.add(mileage)?;
            sets.push(format!("mileage_km = {p}"));
        }
        if let Some(ref color) = req.color {
            let p = pb.add(color.clone())?;
            sets.push(format!("color = {p}"));
        }
        if let Some(doors) = req.doors_count {
            let p = pb.add(doors)?;
            sets.push(format!("doors_count = {p}"));
        }
        if let Some(ref steering) = req.steering {
            let p = pb.add(steering.clone())?;
            sets.push(format!("steering = {p}::steering_side"));
        }
        if let Some(ref condition) = req.condition {
            let p = pb.add(condition.clone())?;
            sets.push(format!("condition = {p}::car_condition"));
        }
        if let Some(ref features) = req.features {
            let json = serde_json::to_value(features)?;
            let p = pb.add(json)?;
            sets.push(format!("features = {p}"));
        }
        if let Some(ref location) = req.location {
            let p = pb.add(location.clone())?;
            sets.push(format!("location = {p}"));
        }

        if sets.is_empty() {
            return Ok(());
        }

        // Always update updated_at
        sets.push("updated_at = NOW()".to_string());

        let id_p = pb.add(id)?;
        let sql = format!(
            "UPDATE listings SET {} WHERE id = {}",
            sets.join(", "),
            id_p,
        );

        sqlx::query_with(&sql, pb.into_args())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn archive_listing(&self, id: Uuid, reason: &str) -> Result<(), anyhow::Error> {
        let status = if reason == "sold" { "sold" } else { "archived" };
        sqlx::query(
            "UPDATE listings SET status = $2::listing_status, removal_reason = $3::removal_reason, removed_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn is_owner(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM listings WHERE id = $1 AND user_id = $2)",
        )
        .bind(listing_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn add_photo(
        &self,
        listing_id: Uuid,
        url: String,
        thumbnail_url: Option<String>,
        sort_order: i16,
        is_primary: bool,
    ) -> Result<Uuid, anyhow::Error> {
        let id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO listing_photos (listing_id, url, thumbnail_url, sort_order, is_primary)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id"#,
        )
        .bind(listing_id)
        .bind(url)
        .bind(thumbnail_url)
        .bind(sort_order)
        .bind(is_primary)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn delete_photo(&self, photo_id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM listing_photos WHERE id = $1")
            .bind(photo_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn count_photos(&self, listing_id: Uuid) -> Result<i64, anyhow::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM listing_photos WHERE listing_id = $1",
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn get_photo_by_id(
        &self,
        photo_id: Uuid,
    ) -> Result<Option<ListingPhoto>, anyhow::Error> {
        let photo = sqlx::query_as::<_, ListingPhoto>(
            r#"SELECT id, listing_id, url, thumbnail_url, sort_order, is_primary, created_at
               FROM listing_photos
               WHERE id = $1"#,
        )
        .bind(photo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(photo)
    }

    async fn get_user_listings(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error> {
        let mut pb = ParamBuilder::new();
        let p_user = pb.add(user_id)?;
        let mut conditions = vec![format!("l.user_id = {p_user}"), "l.status = 'active'".to_string()];

        if let (Some(created_at), Some(id)) = (cursor_created_at, cursor_id) {
            let p1 = pb.add(created_at)?;
            let p2 = pb.add(id)?;
            conditions.push(format!(
                "(l.created_at < {p1} OR (l.created_at = {p1} AND l.id < {p2}))"
            ));
        }

        let where_clause = conditions.join(" AND ");
        let limit_val = limit + 1;
        let p_limit = pb.add(limit_val)?;

        let sql = format!(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                NULL::text AS status, NULL::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM listings l
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE {where_clause}
            ORDER BY l.created_at DESC, l.id DESC
            LIMIT {p_limit}"#,
        );

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, pb.into_args())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn get_user_favorites(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error> {
        let mut pb = ParamBuilder::new();
        let p_user = pb.add(user_id)?;
        let mut conditions = vec![
            format!("f.user_id = {p_user}"),
            "l.status = 'active'".to_string(),
        ];

        if let (Some(created_at), Some(id)) = (cursor_created_at, cursor_id) {
            let p1 = pb.add(created_at)?;
            let p2 = pb.add(id)?;
            conditions.push(format!(
                "(f.created_at < {p1} OR (f.created_at = {p1} AND f.listing_id < {p2}))"
            ));
        }

        let where_clause = conditions.join(" AND ");
        let limit_val = limit + 1;
        let p_limit = pb.add(limit_val)?;

        let sql = format!(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                NULL::text AS status, NULL::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM favorites f
            INNER JOIN listings l ON l.id = f.listing_id
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE {where_clause}
            ORDER BY f.created_at DESC, f.listing_id DESC
            LIMIT {p_limit}"#,
        );

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, pb.into_args())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn get_user_likes(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error> {
        let mut pb = ParamBuilder::new();
        let p_user = pb.add(user_id)?;
        let mut conditions = vec![
            format!("lk.user_id = {p_user}"),
            "l.status = 'active'".to_string(),
        ];

        if let (Some(created_at), Some(id)) = (cursor_created_at, cursor_id) {
            let p1 = pb.add(created_at)?;
            let p2 = pb.add(id)?;
            conditions.push(format!(
                "(lk.created_at < {p1} OR (lk.created_at = {p1} AND lk.listing_id < {p2}))"
            ));
        }

        let where_clause = conditions.join(" AND ");
        let limit_val = limit + 1;
        let p_limit = pb.add(limit_val)?;

        let sql = format!(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                NULL::text AS status, NULL::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM likes lk
            INNER JOIN listings l ON l.id = lk.listing_id
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE {where_clause}
            ORDER BY lk.created_at DESC, lk.listing_id DESC
            LIMIT {p_limit}"#,
        );

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, pb.into_args())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn get_user_archived_listings(
        &self,
        user_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<FeedRow>, anyhow::Error> {
        let mut pb = ParamBuilder::new();
        let p_user = pb.add(user_id)?;
        let mut conditions = vec![
            format!("l.user_id = {p_user}"),
            "l.status IN ('sold', 'archived')".to_string(),
        ];

        if let (Some(removed_at), Some(id)) = (cursor_created_at, cursor_id) {
            let p1 = pb.add(removed_at)?;
            let p2 = pb.add(id)?;
            conditions.push(format!(
                "(l.removed_at < {p1} OR (l.removed_at = {p1} AND l.id < {p2}))"
            ));
        }

        let where_clause = conditions.join(" AND ");
        let limit_val = limit + 1;
        let p_limit = pb.add(limit_val)?;

        let sql = format!(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                l.status::text AS status, l.removal_reason::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM listings l
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE {where_clause}
            ORDER BY l.removed_at DESC NULLS LAST, l.id DESC
            LIMIT {p_limit}"#,
        );

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, pb.into_args())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn count_user_listings(&self, user_id: Uuid) -> Result<i64, anyhow::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM listings WHERE user_id = $1 AND status = 'active'",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn boost_listing(&self, listing_id: Uuid, user_id: Uuid) -> Result<(), anyhow::Error> {
        let rows_affected = sqlx::query(
            "UPDATE listings SET boosted_at = NOW(), updated_at = NOW() WHERE id = $1 AND user_id = $2 AND status = 'active'"
        )
        .bind(listing_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            anyhow::bail!("Listing not found or not owned by user");
        }
        Ok(())
    }

    async fn add_promoted_stars(&self, listing_id: Uuid, user_id: Uuid, stars: i32) -> Result<i32, anyhow::Error> {
        let new_total: i32 = sqlx::query_scalar(
            "UPDATE listings SET promoted_stars = promoted_stars + $3, updated_at = NOW() WHERE id = $1 AND user_id = $2 AND status = 'active' RETURNING promoted_stars"
        )
        .bind(listing_id)
        .bind(user_id)
        .bind(stars)
        .fetch_one(&self.pool)
        .await?;
        Ok(new_total)
    }

    async fn get_promoted(&self, _user_id: Option<Uuid>, limit: i64) -> Result<Vec<FeedRow>, anyhow::Error> {
        let mut pb = ParamBuilder::new();
        let limit_val = limit;
        let p_limit = pb.add(limit_val)?;

        let sql = format!(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                NULL::text AS status, NULL::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM listings l
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE l.promoted_stars > 0 AND l.status = 'active'
            ORDER BY l.promoted_stars DESC, l.id DESC
            LIMIT {p_limit}"#,
        );

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, pb.into_args())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn decay_promoted_stars(&self, amount: i32) -> Result<u64, anyhow::Error> {
        let result = sqlx::query(
            r#"UPDATE listings
               SET promoted_stars = GREATEST(0, promoted_stars - $1),
                   updated_at = NOW()
               WHERE promoted_stars > 0 AND status = 'active'"#,
        )
        .bind(amount)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn extend_listing(&self, listing_id: Uuid, user_id: Uuid) -> Result<(), anyhow::Error> {
        let result = sqlx::query(
            r#"UPDATE listings
               SET expires_at = now() + INTERVAL '30 days'
               WHERE id = $1 AND user_id = $2 AND status = 'active'"#,
        )
        .bind(listing_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Listing not found or not owned by user"));
        }

        Ok(())
    }

    async fn feed_by_ids(&self, ids: &[Uuid]) -> Result<Vec<FeedRow>, anyhow::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query_as::<_, FeedRow>(
            r#"SELECT
                l.id, l.title, l.price, l.currency::text AS currency,
                l.year, l.mileage_km,
                l.fuel::text AS fuel, l.transmission::text AS transmission,
                l.body::text AS body, l.drive::text AS drive,
                l.horsepower, l.location,
                l.views_count, l.likes_count, l.created_at,
                m.id AS make_id, m.name AS make_name, m.slug AS make_slug,
                cm.id AS model_id, cm.name AS model_name, cm.slug AS model_slug,
                ph.id AS photo_id, ph.url AS photo_url, ph.thumbnail_url AS photo_thumbnail_url,
                NULL::text AS status, NULL::text AS removal_reason,
                l.promoted_stars, l.boosted_at
            FROM listings l
            INNER JOIN car_makes m ON m.id = l.make_id
            INNER JOIN car_models cm ON cm.id = l.model_id
            LEFT JOIN LATERAL (
                SELECT lp.id, lp.url, lp.thumbnail_url
                FROM listing_photos lp
                WHERE lp.listing_id = l.id AND lp.is_primary = TRUE
                LIMIT 1
            ) ph ON TRUE
            WHERE l.id = ANY($1) AND l.status = 'active'
            ORDER BY array_position($1, l.id)"#,
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn get_listing_attrs(
        &self,
        id: Uuid,
    ) -> Result<Option<ListingAttrsRow>, anyhow::Error> {
        let row = sqlx::query_as::<_, ListingAttrsRow>(
            r#"SELECT make_id, model_id, body::text AS body, fuel::text AS fuel,
                      transmission::text AS transmission, drive::text AS drive,
                      price, year
               FROM listings WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }
}
