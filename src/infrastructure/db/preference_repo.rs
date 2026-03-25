use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::preference_repository::PreferenceRepository;
use crate::domain::user_preference::UserPreference;

pub struct PgPreferenceRepository {
    pool: PgPool,
}

impl PgPreferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PrefRow {
    user_id: Uuid,
    make_weights: serde_json::Value,
    model_weights: serde_json::Value,
    body_weights: serde_json::Value,
    fuel_weights: serde_json::Value,
    trans_weights: serde_json::Value,
    drive_weights: serde_json::Value,
    price_center: Option<f64>,
    year_center: Option<f64>,
    total_interactions: i32,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<PrefRow> for UserPreference {
    fn from(r: PrefRow) -> Self {
        Self {
            user_id: r.user_id,
            make_weights: serde_json::from_value(r.make_weights).unwrap_or_default(),
            model_weights: serde_json::from_value(r.model_weights).unwrap_or_default(),
            body_weights: serde_json::from_value(r.body_weights).unwrap_or_default(),
            fuel_weights: serde_json::from_value(r.fuel_weights).unwrap_or_default(),
            trans_weights: serde_json::from_value(r.trans_weights).unwrap_or_default(),
            drive_weights: serde_json::from_value(r.drive_weights).unwrap_or_default(),
            price_center: r.price_center,
            year_center: r.year_center,
            total_interactions: r.total_interactions,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ScoredRow {
    id: Uuid,
}

#[async_trait]
impl PreferenceRepository for PgPreferenceRepository {
    async fn get(&self, user_id: Uuid) -> Result<Option<UserPreference>, anyhow::Error> {
        let row = sqlx::query_as::<_, PrefRow>(
            r#"SELECT user_id, make_weights, model_weights, body_weights, fuel_weights,
                      trans_weights, drive_weights, price_center, year_center,
                      total_interactions, updated_at
               FROM user_preferences WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn upsert(&self, pref: &UserPreference) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"INSERT INTO user_preferences
               (user_id, make_weights, model_weights, body_weights, fuel_weights,
                trans_weights, drive_weights, price_center, year_center,
                total_interactions, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
               ON CONFLICT (user_id) DO UPDATE SET
                 make_weights = $2, model_weights = $3, body_weights = $4,
                 fuel_weights = $5, trans_weights = $6, drive_weights = $7,
                 price_center = $8, year_center = $9,
                 total_interactions = $10, updated_at = now()"#,
        )
        .bind(pref.user_id)
        .bind(serde_json::to_value(&pref.make_weights)?)
        .bind(serde_json::to_value(&pref.model_weights)?)
        .bind(serde_json::to_value(&pref.body_weights)?)
        .bind(serde_json::to_value(&pref.fuel_weights)?)
        .bind(serde_json::to_value(&pref.trans_weights)?)
        .bind(serde_json::to_value(&pref.drive_weights)?)
        .bind(pref.price_center)
        .bind(pref.year_center)
        .bind(pref.total_interactions)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn scored_feed(
        &self,
        user_id: Uuid,
        pref: &UserPreference,
    ) -> Result<Vec<Uuid>, anyhow::Error> {
        let rows = sqlx::query_as::<_, ScoredRow>(
            r#"SELECT l.id
               FROM listings l
               WHERE l.status = 'active' AND l.user_id != $9
               ORDER BY
                 (
                   COALESCE(($1::jsonb ->> l.make_id::text)::float, 0) * 2.0
                   + COALESCE(($2::jsonb ->> l.model_id::text)::float, 0) * 3.0
                   + COALESCE(($3::jsonb ->> l.body::text)::float, 0)
                   + COALESCE(($4::jsonb ->> l.fuel::text)::float, 0)
                   + COALESCE(($5::jsonb ->> l.transmission::text)::float, 0)
                   + COALESCE(($6::jsonb ->> COALESCE(l.drive::text, ''))::float, 0)
                   + CASE WHEN $7::double precision IS NOT NULL THEN
                       GREATEST(0, 1.0 - ABS(l.price::float - $7::double precision) / GREATEST($7::double precision, 1)) * 0.5
                     ELSE 0 END
                   + CASE WHEN $8::double precision IS NOT NULL THEN
                       GREATEST(0, 1.0 - ABS(l.year::float - $8::double precision) / 10.0) * 0.5
                     ELSE 0 END
                   + CASE WHEN l.promoted_stars > 0 THEN l.promoted_stars::float * 0.5 ELSE 0 END
                 ) DESC,
                 l.created_at DESC,
                 l.id DESC"#,
        )
        .bind(serde_json::to_value(&pref.make_weights)?)
        .bind(serde_json::to_value(&pref.model_weights)?)
        .bind(serde_json::to_value(&pref.body_weights)?)
        .bind(serde_json::to_value(&pref.fuel_weights)?)
        .bind(serde_json::to_value(&pref.trans_weights)?)
        .bind(serde_json::to_value(&pref.drive_weights)?)
        .bind(pref.price_center)
        .bind(pref.year_center)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}
