use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::dto::car::*;
use crate::application::ports::car_repository::{CarRepository, FeedFilter, FeedRow};
use crate::application::ports::preference_repository::PreferenceRepository;
use crate::application::services::preference_service::PreferenceService;
use crate::domain::car::ALLOWED_FEATURES;
use crate::domain::user_preference::{InteractionSignal, ListingAttributes};
use crate::infrastructure::redis::feed_cache::FeedCache;
use crate::infrastructure::redis::view_counter::ViewCounter;

/// Cursor payload encoded as base64(JSON).
#[derive(Debug, Serialize, Deserialize)]
struct CursorData {
    /// created_at (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    c: Option<String>,
    /// id (uuid)
    i: String,
    /// price
    #[serde(skip_serializing_if = "Option::is_none")]
    p: Option<i32>,
    /// mileage
    #[serde(skip_serializing_if = "Option::is_none")]
    m: Option<i32>,
}

/// Personalized feed cursor (offset-based, paginating from Redis snapshot).
#[derive(Debug, Serialize, Deserialize)]
struct PersonalizedCursor {
    /// "p" marker for personalized
    t: String,
    /// offset into the cached ID list
    o: usize,
}

pub struct CarService {
    car_repo: Arc<dyn CarRepository>,
    view_counter: Arc<ViewCounter>,
    preference_service: Arc<PreferenceService>,
    feed_cache: Arc<FeedCache>,
    preference_repo: Arc<dyn PreferenceRepository>,
}

impl CarService {
    pub fn new(
        car_repo: Arc<dyn CarRepository>,
        view_counter: Arc<ViewCounter>,
        preference_service: Arc<PreferenceService>,
        feed_cache: Arc<FeedCache>,
        preference_repo: Arc<dyn PreferenceRepository>,
    ) -> Self {
        Self {
            car_repo,
            view_counter,
            preference_service,
            feed_cache,
            preference_repo,
        }
    }

    /// Expose a reference to the car repository (for photo operations in handlers).
    pub fn car_repo_ref(&self) -> &dyn CarRepository {
        self.car_repo.as_ref()
    }

    /// Parse comma-separated values into a Vec, filtering empty strings.
    fn parse_csv(val: &Option<String>) -> Vec<String> {
        match val {
            Some(s) if !s.is_empty() => s.split(',').map(|v| v.trim().to_string()).collect(),
            _ => vec![],
        }
    }

    fn decode_cursor(encoded: &str) -> Result<CursorData, anyhow::Error> {
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .context("Invalid cursor encoding")?;
        let data: CursorData =
            serde_json::from_slice(&bytes).context("Invalid cursor payload")?;
        Ok(data)
    }

    fn encode_cursor(data: &CursorData) -> String {
        let json = serde_json::to_string(data).expect("cursor serialization cannot fail");
        URL_SAFE_NO_PAD.encode(json.as_bytes())
    }

    fn encode_personalized_cursor(offset: usize) -> String {
        let json = serde_json::to_string(&PersonalizedCursor {
            t: "p".to_string(),
            o: offset,
        })
        .expect("cursor serialization cannot fail");
        URL_SAFE_NO_PAD.encode(json.as_bytes())
    }

    fn try_decode_personalized_cursor(encoded: &str) -> Option<PersonalizedCursor> {
        let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
        let cursor: PersonalizedCursor = serde_json::from_slice(&bytes).ok()?;
        if cursor.t == "p" {
            Some(cursor)
        } else {
            None
        }
    }

    fn has_filters(query: &FeedQuery) -> bool {
        query.make_id.is_some()
            || query.model_id.is_some()
            || query.year_min.is_some()
            || query.year_max.is_some()
            || query.price_min.is_some()
            || query.price_max.is_some()
            || query.mileage_min.is_some()
            || query.mileage_max.is_some()
            || query.fuel.as_ref().map_or(false, |s| !s.is_empty())
            || query.body.as_ref().map_or(false, |s| !s.is_empty())
            || query.transmission.as_ref().map_or(false, |s| !s.is_empty())
            || query.drive.as_ref().map_or(false, |s| !s.is_empty())
            || query.features.as_ref().map_or(false, |s| !s.is_empty())
    }

    /// Get the feed with pagination and filters.
    pub async fn get_feed(
        &self,
        query: &FeedQuery,
        user_id: Option<Uuid>,
    ) -> Result<FeedResponse, anyhow::Error> {
        let sort = query.sort.clone().unwrap_or_else(|| "newest".to_string());
        let limit = query.limit.unwrap_or(20).min(50).max(1);

        // Check for personalized cursor continuation
        if let Some(cursor_str) = &query.cursor {
            if let Some(pc) = Self::try_decode_personalized_cursor(cursor_str) {
                return self
                    .get_personalized_feed_page(user_id, pc.o, limit as usize)
                    .await;
            }
        }

        // Try personalized feed on first page (no cursor, no filters, newest sort, authenticated)
        let use_personalized = query.cursor.is_none()
            && !Self::has_filters(query)
            && (sort == "newest" || sort.is_empty())
            && user_id.is_some();

        if use_personalized {
            let uid = user_id.unwrap();
            if let Some(resp) = self
                .try_build_personalized_feed(uid, limit as usize)
                .await?
            {
                return Ok(resp);
            }
            // Cold start or error → fall through to standard feed
        }

        // Standard feed logic
        let cursor = match &query.cursor {
            Some(c) if !c.is_empty() => Some(Self::decode_cursor(c)?),
            _ => None,
        };

        let filter = FeedFilter {
            cursor_created_at: cursor.as_ref().and_then(|c| {
                c.c.as_ref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            cursor_id: cursor
                .as_ref()
                .and_then(|c| Uuid::parse_str(&c.i).ok()),
            cursor_price: cursor.as_ref().and_then(|c| c.p),
            cursor_mileage: cursor.as_ref().and_then(|c| c.m),
            limit,
            sort: sort.clone(),
            make_id: query.make_id,
            model_id: query.model_id,
            year_min: query.year_min,
            year_max: query.year_max,
            price_min: query.price_min,
            price_max: query.price_max,
            mileage_min: query.mileage_min,
            mileage_max: query.mileage_max,
            fuel_types: Self::parse_csv(&query.fuel),
            body_types: Self::parse_csv(&query.body),
            transmissions: Self::parse_csv(&query.transmission),
            drive_types: Self::parse_csv(&query.drive),
            features: Self::parse_csv(&query.features),
        };

        let mut rows = self.car_repo.feed(&filter).await?;

        let has_more = rows.len() as i64 > limit;
        if has_more {
            rows.truncate(limit as usize);
        }

        // Build next cursor from last row
        let next_cursor = if has_more {
            rows.last().map(|row| {
                let cursor_time = if sort == "newest" || sort.is_empty() {
                    row.boosted_at.unwrap_or(row.created_at)
                } else {
                    row.created_at
                };
                let data = CursorData {
                    c: Some(cursor_time.to_rfc3339()),
                    i: row.id.to_string(),
                    p: Some(row.price),
                    m: Some(row.mileage_km),
                };
                Self::encode_cursor(&data)
            })
        } else {
            None
        };

        // Batch fetch liked/favorited IDs for authenticated user
        let listing_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let (liked_ids, favorited_ids) = if let Some(uid) = user_id {
            let liked = self.car_repo.get_liked_ids(uid, &listing_ids).await?;
            let favorited = self.car_repo.get_favorited_ids(uid, &listing_ids).await?;
            (
                liked.into_iter().collect::<HashSet<_>>(),
                favorited.into_iter().collect::<HashSet<_>>(),
            )
        } else {
            (HashSet::new(), HashSet::new())
        };

        let items: Vec<CarSummaryResponse> = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked_ids, &favorited_ids))
            .collect();

        // Fetch promoted listings on first page only (standard feed)
        let promoted = if query.cursor.is_none() {
            let promoted_rows = self.car_repo.get_promoted(user_id, 10).await?;
            let promo_ids: Vec<Uuid> = promoted_rows.iter().map(|r| r.id).collect();
            let (promo_liked, promo_fav) = if let Some(uid) = user_id {
                (
                    self.car_repo.get_liked_ids(uid, &promo_ids).await?.into_iter().collect::<HashSet<_>>(),
                    self.car_repo.get_favorited_ids(uid, &promo_ids).await?.into_iter().collect::<HashSet<_>>(),
                )
            } else {
                (HashSet::new(), HashSet::new())
            };
            promoted_rows.into_iter()
                .map(|row| self.feed_row_to_summary(row, &promo_liked, &promo_fav))
                .collect()
        } else {
            vec![]
        };

        Ok(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted,
        })
    }

    /// Try to build a personalized feed (first page). Returns None if cold start.
    async fn try_build_personalized_feed(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Option<FeedResponse>, anyhow::Error> {
        let pref = match self.preference_repo.get(user_id).await? {
            Some(p) if !p.is_cold_start() => p,
            _ => return Ok(None),
        };

        let scored_ids = self.preference_repo.scored_feed(user_id, &pref).await?;
        if scored_ids.is_empty() {
            return Ok(None);
        }

        // Cache the full snapshot
        let _ = self
            .feed_cache
            .set_feed_snapshot(user_id, &scored_ids)
            .await;

        // Take first page
        let has_more = scored_ids.len() > limit;
        let page_ids: Vec<Uuid> = scored_ids.into_iter().take(limit).collect();
        let rows = self.car_repo.feed_by_ids(&page_ids).await?;

        let listing_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let liked = self
            .car_repo
            .get_liked_ids(user_id, &listing_ids)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let favorited = self
            .car_repo
            .get_favorited_ids(user_id, &listing_ids)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();

        let items = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked, &favorited))
            .collect();

        let next_cursor = if has_more {
            Some(Self::encode_personalized_cursor(limit))
        } else {
            None
        };

        Ok(Some(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted: vec![], // Promoted integrated into scoring
        }))
    }

    /// Get a subsequent page from the personalized feed snapshot.
    async fn get_personalized_feed_page(
        &self,
        user_id: Option<Uuid>,
        offset: usize,
        limit: usize,
    ) -> Result<FeedResponse, anyhow::Error> {
        let uid = user_id.ok_or_else(|| anyhow::anyhow!("Auth required for personalized feed"))?;

        let cached = self.feed_cache.get_feed_page(uid, offset, limit).await?;
        let (page_ids, has_more) = match cached {
            Some(result) => result,
            None => {
                // Cache expired — return empty page, frontend will refresh
                return Ok(FeedResponse {
                    items: vec![],
                    cursor: None,
                    has_more: false,
                    promoted: vec![],
                });
            }
        };

        let rows = self.car_repo.feed_by_ids(&page_ids).await?;

        let listing_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let liked = self
            .car_repo
            .get_liked_ids(uid, &listing_ids)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let favorited = self
            .car_repo
            .get_favorited_ids(uid, &listing_ids)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();

        let items = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked, &favorited))
            .collect();

        let next_cursor = if has_more {
            Some(Self::encode_personalized_cursor(offset + limit))
        } else {
            None
        };

        Ok(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted: vec![],
        })
    }

    fn feed_row_to_summary(
        &self,
        row: FeedRow,
        liked_ids: &HashSet<Uuid>,
        favorited_ids: &HashSet<Uuid>,
    ) -> CarSummaryResponse {
        let photo = row.photo_id.map(|id| PhotoResponse {
            id: id.to_string(),
            url: row.photo_url.unwrap_or_default(),
            thumbnail_url: row.photo_thumbnail_url,
            sort_order: 0,
            is_primary: true,
        });

        CarSummaryResponse {
            id: row.id.to_string(),
            title: row.title,
            price: row.price,
            currency: row.currency,
            year: row.year,
            mileage_km: row.mileage_km,
            fuel: row.fuel,
            transmission: row.transmission,
            body: row.body,
            drive: row.drive,
            horsepower: row.horsepower,
            location: row.location,
            make: MakeResponse {
                id: row.make_id,
                name: row.make_name,
                slug: row.make_slug,
            },
            model: ModelResponse {
                id: row.model_id,
                name: row.model_name,
                slug: row.model_slug,
            },
            photo,
            views_count: row.views_count,
            likes_count: row.likes_count,
            is_liked: liked_ids.contains(&row.id),
            is_favorited: favorited_ids.contains(&row.id),
            created_at: row.created_at.to_rfc3339(),
            status: row.status,
            removal_reason: row.removal_reason,
            promoted_stars: if row.promoted_stars > 0 { Some(row.promoted_stars) } else { None },
            boosted_at: row.boosted_at.map(|dt| dt.to_rfc3339()),
        }
    }

    /// Get listing detail by ID. Increments view count if user is authenticated.
    pub async fn get_listing(
        &self,
        id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Option<CarDetailResponse>, anyhow::Error> {
        let row = match self.car_repo.find_by_id(id).await? {
            Some(r) => r,
            None => return Ok(None),
        };

        let is_owner = user_id.map_or(false, |uid| uid == row.user_id);
        let is_removed = row.status == "sold" || row.status == "archived";

        // Record view only for non-removed listings (or owner viewing own)
        if !(is_removed && !is_owner) {
            if let Some(uid) = user_id {
                let is_new = self
                    .view_counter
                    .record_view(&uid.to_string(), &id.to_string())
                    .await
                    .unwrap_or(false);
                if is_new {
                    let _ = self.car_repo.increment_views(id).await;

                    // Fire-and-forget preference update
                    let pref_svc = self.preference_service.clone();
                    let attrs = ListingAttributes {
                        make_id: row.make_id,
                        model_id: row.model_id,
                        body: row.body.clone(),
                        fuel: row.fuel.clone(),
                        transmission: row.transmission.clone(),
                        drive: row.drive.clone(),
                        price: row.price,
                        year: row.year,
                    };
                    tokio::spawn(async move {
                        let _ = pref_svc
                            .record_interaction(uid, &attrs, InteractionSignal::View)
                            .await;
                    });
                }
            }
        }

        // Get photos
        let photos = self.car_repo.get_photos(id).await?;

        // Get liked/favorited status
        let (is_liked, is_favorited) = if let Some(uid) = user_id {
            let liked = self.car_repo.is_liked(uid, id).await?;
            let fav = self.car_repo.is_favorited(uid, id).await?;
            (liked, fav)
        } else {
            (false, false)
        };

        // Parse features from JSON
        let features: Vec<String> = match &row.features {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };

        let photo_responses: Vec<PhotoResponse> = photos
            .into_iter()
            .map(|p| PhotoResponse {
                id: p.id.to_string(),
                url: p.url,
                thumbnail_url: p.thumbnail_url,
                sort_order: p.sort_order,
                is_primary: p.is_primary,
            })
            .collect();

        let mut response = CarDetailResponse {
            id: row.id.to_string(),
            title: row.title,
            description: row.description,
            price: Some(row.price),
            currency: row.currency,
            status: row.status,
            year: row.year,
            mileage_km: row.mileage_km,
            fuel: row.fuel,
            transmission: row.transmission,
            body: row.body,
            drive: row.drive,
            engine_displacement_cc: row.engine_displacement_cc,
            horsepower: row.horsepower,
            color: row.color,
            doors_count: row.doors_count,
            steering: row.steering,
            condition: row.condition,
            features,
            location: row.location,
            make: MakeResponse {
                id: row.make_id,
                name: row.make_name,
                slug: row.make_slug,
            },
            model: ModelResponse {
                id: row.model_id,
                name: row.model_name,
                slug: row.model_slug,
            },
            photos: photo_responses,
            seller: Some(SellerResponse {
                id: row.user_id.to_string(),
                username: row.seller_username,
                first_name: row.seller_first_name,
                avatar_url: row.seller_avatar_url,
                telegram_id: row.seller_telegram_id,
            }),
            views_count: row.views_count,
            likes_count: row.likes_count,
            is_liked,
            is_favorited,
            is_owner,
            created_at: Some(row.created_at.to_rfc3339()),
            updated_at: Some(row.updated_at.to_rfc3339()),
            removal_reason: row.removal_reason,
        };

        // Strip sensitive data for non-owner viewing removed listings
        if is_removed && !is_owner {
            response.price = None;
            response.seller = None;
            response.created_at = None;
            response.updated_at = None;
            // Keep only primary photo (or first if no primary)
            if let Some(primary) = response.photos.iter().find(|p| p.is_primary).cloned() {
                response.photos = vec![primary];
            } else if !response.photos.is_empty() {
                response.photos = vec![response.photos.swap_remove(0)];
            }
        }

        Ok(Some(response))
    }

    /// Toggle like on a listing. Returns (is_liked, new_count).
    pub async fn toggle_like(
        &self,
        user_id: Uuid,
        listing_id: Uuid,
    ) -> Result<LikeResponse, anyhow::Error> {
        let (liked, likes_count) = self.car_repo.toggle_like(user_id, listing_id).await?;

        // Fire-and-forget preference update
        let pref_svc = self.preference_service.clone();
        let car_repo = self.car_repo.clone();
        let signal = if liked {
            InteractionSignal::Like
        } else {
            InteractionSignal::Unlike
        };
        tokio::spawn(async move {
            if let Ok(Some(row)) = car_repo.get_listing_attrs(listing_id).await {
                let attrs = ListingAttributes {
                    make_id: row.make_id,
                    model_id: row.model_id,
                    body: row.body,
                    fuel: row.fuel,
                    transmission: row.transmission,
                    drive: row.drive,
                    price: row.price,
                    year: row.year,
                };
                let _ = pref_svc.record_interaction(user_id, &attrs, signal).await;
            }
        });

        Ok(LikeResponse { liked, likes_count })
    }

    /// Toggle favorite on a listing.
    pub async fn toggle_favorite(
        &self,
        user_id: Uuid,
        listing_id: Uuid,
    ) -> Result<FavoriteResponse, anyhow::Error> {
        let favorited = self.car_repo.toggle_favorite(user_id, listing_id).await?;

        // Fire-and-forget preference update
        let pref_svc = self.preference_service.clone();
        let car_repo = self.car_repo.clone();
        let signal = if favorited {
            InteractionSignal::Favorite
        } else {
            InteractionSignal::Unfavorite
        };
        tokio::spawn(async move {
            if let Ok(Some(row)) = car_repo.get_listing_attrs(listing_id).await {
                let attrs = ListingAttributes {
                    make_id: row.make_id,
                    model_id: row.model_id,
                    body: row.body,
                    fuel: row.fuel,
                    transmission: row.transmission,
                    drive: row.drive,
                    price: row.price,
                    year: row.year,
                };
                let _ = pref_svc.record_interaction(user_id, &attrs, signal).await;
            }
        });

        Ok(FavoriteResponse { favorited })
    }

    /// Get all car makes.
    pub async fn get_makes(&self) -> Result<Vec<MakeResponse>, anyhow::Error> {
        let makes = self.car_repo.get_makes().await?;
        Ok(makes
            .into_iter()
            .map(|m| MakeResponse {
                id: m.id,
                name: m.name,
                slug: m.slug,
            })
            .collect())
    }

    /// Get models for a specific make.
    pub async fn get_models(&self, make_id: i32) -> Result<Vec<ModelResponse>, anyhow::Error> {
        let models = self.car_repo.get_models_by_make(make_id).await?;
        Ok(models
            .into_iter()
            .map(|m| ModelResponse {
                id: m.id,
                name: m.name,
                slug: m.slug,
            })
            .collect())
    }

    /// Create a new listing. Title is auto-generated as "Make Model, Year".
    pub async fn create_listing(
        &self,
        user_id: Uuid,
        req: &CreateListingRequest,
    ) -> Result<CarDetailResponse, anyhow::Error> {
        // Auto-generate title from make + model + year
        let makes = self.car_repo.get_makes().await?;
        let models = self.car_repo.get_models_by_make(req.make_id).await?;
        let make_name = makes.iter().find(|m| m.id == req.make_id).map(|m| m.name.as_str()).unwrap_or("Auto");
        let model_name = models.iter().find(|m| m.id == req.model_id).map(|m| m.name.as_str()).unwrap_or("");
        let mut req = req.clone();
        req.title = format!("{} {}, {}", make_name, model_name, req.year);

        let listing_id = self.car_repo.create_listing(user_id, &req).await?;
        // Return the full detail response
        self.get_listing(listing_id, Some(user_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Listing was created but could not be fetched"))
    }

    /// Update a listing. Validates ownership.
    pub async fn update_listing(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
        req: &UpdateListingRequest,
    ) -> Result<(), anyhow::Error> {
        let is_owner = self.car_repo.is_owner(listing_id, user_id).await?;
        if !is_owner {
            return Err(anyhow::anyhow!("FORBIDDEN"));
        }

        self.car_repo.update_listing(listing_id, req).await?;
        Ok(())
    }

    /// Archive a listing. Validates ownership.
    pub async fn archive_listing(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
        reason: &str,
    ) -> Result<(), anyhow::Error> {
        let is_owner = self.car_repo.is_owner(listing_id, user_id).await?;
        if !is_owner {
            return Err(anyhow::anyhow!("FORBIDDEN"));
        }

        self.car_repo.archive_listing(listing_id, reason).await?;
        Ok(())
    }

    /// Check if a user owns a listing.
    pub async fn check_ownership(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        self.car_repo.is_owner(listing_id, user_id).await
    }

    /// Get user's own listings with cursor pagination.
    pub async fn get_user_listings(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
        cursor: Option<String>,
        limit: Option<i64>,
    ) -> Result<FeedResponse, anyhow::Error> {
        let limit = limit.unwrap_or(20).min(50).max(1);

        let (cursor_created_at, cursor_id) = match &cursor {
            Some(c) if !c.is_empty() => {
                let data = Self::decode_cursor(c)?;
                let created_at = data.c.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                });
                let id = Uuid::parse_str(&data.i).ok();
                (created_at, id)
            }
            _ => (None, None),
        };

        let mut rows = self
            .car_repo
            .get_user_listings(user_id, cursor_created_at, cursor_id, limit)
            .await?;

        let has_more = rows.len() as i64 > limit;
        if has_more {
            rows.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            rows.last().map(|row| {
                Self::encode_cursor(&CursorData {
                    c: Some(row.created_at.to_rfc3339()),
                    i: row.id.to_string(),
                    p: None,
                    m: None,
                })
            })
        } else {
            None
        };

        // Fetch liked/favorited based on viewer (may differ from listing owner)
        let listing_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let (liked_ids, favorited_ids) = if let Some(vid) = viewer_id {
            let liked = self.car_repo.get_liked_ids(vid, &listing_ids).await?;
            let favorited = self.car_repo.get_favorited_ids(vid, &listing_ids).await?;
            (
                liked.into_iter().collect::<HashSet<_>>(),
                favorited.into_iter().collect::<HashSet<_>>(),
            )
        } else {
            (HashSet::new(), HashSet::new())
        };

        let items = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked_ids, &favorited_ids))
            .collect();

        Ok(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted: vec![],
        })
    }

    /// Get user's favorited listings with cursor pagination.
    pub async fn get_user_favorites(
        &self,
        user_id: Uuid,
        cursor: Option<String>,
        limit: Option<i64>,
    ) -> Result<FeedResponse, anyhow::Error> {
        let limit = limit.unwrap_or(20).min(50).max(1);

        let (cursor_created_at, cursor_id) = match &cursor {
            Some(c) if !c.is_empty() => {
                let data = Self::decode_cursor(c)?;
                let created_at = data.c.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                });
                let id = Uuid::parse_str(&data.i).ok();
                (created_at, id)
            }
            _ => (None, None),
        };

        let mut rows = self
            .car_repo
            .get_user_favorites(user_id, cursor_created_at, cursor_id, limit)
            .await?;

        let has_more = rows.len() as i64 > limit;
        if has_more {
            rows.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            rows.last().map(|row| {
                Self::encode_cursor(&CursorData {
                    c: Some(row.created_at.to_rfc3339()),
                    i: row.id.to_string(),
                    p: None,
                    m: None,
                })
            })
        } else {
            None
        };

        // All favorites are favorited by definition, but check likes
        let listing_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let liked_ids = self
            .car_repo
            .get_liked_ids(user_id, &listing_ids)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let favorited_ids = listing_ids.iter().copied().collect::<HashSet<_>>();

        let items = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked_ids, &favorited_ids))
            .collect();

        Ok(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted: vec![],
        })
    }

    /// Get user's liked listings with cursor pagination.
    pub async fn get_user_likes(
        &self,
        user_id: Uuid,
        cursor: Option<String>,
        limit: Option<i64>,
    ) -> Result<FeedResponse, anyhow::Error> {
        let limit = limit.unwrap_or(20).min(50).max(1);

        let (cursor_created_at, cursor_id) = match &cursor {
            Some(c) if !c.is_empty() => {
                let data = Self::decode_cursor(c)?;
                let created_at = data.c.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                });
                let id = Uuid::parse_str(&data.i).ok();
                (created_at, id)
            }
            _ => (None, None),
        };

        let mut rows = self
            .car_repo
            .get_user_likes(user_id, cursor_created_at, cursor_id, limit)
            .await?;

        let has_more = rows.len() as i64 > limit;
        if has_more {
            rows.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            rows.last().map(|row| {
                Self::encode_cursor(&CursorData {
                    c: Some(row.created_at.to_rfc3339()),
                    i: row.id.to_string(),
                    p: None,
                    m: None,
                })
            })
        } else {
            None
        };

        // All likes are liked by definition, but check favorites
        let listing_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let liked_ids = listing_ids.iter().copied().collect::<HashSet<_>>();
        let favorited_ids = self
            .car_repo
            .get_favorited_ids(user_id, &listing_ids)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();

        let items = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked_ids, &favorited_ids))
            .collect();

        Ok(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted: vec![],
        })
    }

    /// Get user's archived listings with cursor pagination.
    pub async fn get_user_archived(
        &self,
        user_id: Uuid,
        cursor: Option<String>,
        limit: Option<i64>,
    ) -> Result<FeedResponse, anyhow::Error> {
        let limit = limit.unwrap_or(20).min(50).max(1);

        let (cursor_created_at, cursor_id) = match &cursor {
            Some(c) if !c.is_empty() => {
                let data = Self::decode_cursor(c)?;
                let created_at = data.c.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                });
                let id = Uuid::parse_str(&data.i).ok();
                (created_at, id)
            }
            _ => (None, None),
        };

        let mut rows = self
            .car_repo
            .get_user_archived_listings(user_id, cursor_created_at, cursor_id, limit)
            .await?;

        let has_more = rows.len() as i64 > limit;
        if has_more {
            rows.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            rows.last().map(|row| {
                Self::encode_cursor(&CursorData {
                    c: Some(row.created_at.to_rfc3339()),
                    i: row.id.to_string(),
                    p: None,
                    m: None,
                })
            })
        } else {
            None
        };

        // Own archived listings — no need to check liked/favorited
        let liked_ids = HashSet::new();
        let favorited_ids = HashSet::new();

        let items = rows
            .into_iter()
            .map(|row| self.feed_row_to_summary(row, &liked_ids, &favorited_ids))
            .collect();

        Ok(FeedResponse {
            items,
            cursor: next_cursor,
            has_more,
            promoted: vec![],
        })
    }

    /// Extend a listing's expiration by 30 days. Validates ownership.
    pub async fn extend_listing(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        self.car_repo.extend_listing(listing_id, user_id).await
    }

    /// Count active listings for a user.
    pub async fn count_user_listings(&self, user_id: Uuid) -> Result<i64, anyhow::Error> {
        self.car_repo.count_user_listings(user_id).await
    }

    /// Get available filter options (enum values).
    pub fn get_filter_options(&self) -> FilterOptionsResponse {
        FilterOptionsResponse {
            fuel_types: vec![
                "petrol",
                "diesel",
                "gas_methane",
                "gas_propane",
                "petrol_gas_methane",
                "petrol_gas_propane",
                "electric",
                "hybrid",
            ],
            body_types: vec![
                "sedan",
                "hatchback",
                "wagon",
                "suv",
                "coupe",
                "minivan",
                "pickup",
                "convertible",
                "van",
            ],
            transmission_types: vec!["manual", "automatic", "cvt", "robot"],
            drive_types: vec!["fwd", "rwd", "awd"],
            features: ALLOWED_FEATURES.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::car::{CreateListingRequest, UpdateListingRequest};
    use crate::application::ports::car_repository::{DetailRow, FeedRow, ListingAttrsRow};
    use crate::domain::car::{CarMake, CarModel, ListingPhoto};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};

    struct MockCarRepo {
        feed_rows: Vec<FeedRow>,
    }

    impl MockCarRepo {
        fn new() -> Self {
            Self {
                feed_rows: vec![],
            }
        }

        fn with_feed_rows(mut self, rows: Vec<FeedRow>) -> Self {
            self.feed_rows = rows;
            self
        }
    }

    #[async_trait]
    impl CarRepository for MockCarRepo {
        async fn feed(&self, _filter: &FeedFilter) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(self.feed_rows.clone())
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<DetailRow>, anyhow::Error> {
            Ok(None)
        }
        async fn get_photos(&self, _id: Uuid) -> Result<Vec<ListingPhoto>, anyhow::Error> {
            Ok(vec![])
        }
        async fn get_makes(&self) -> Result<Vec<CarMake>, anyhow::Error> {
            Ok(vec![])
        }
        async fn get_models_by_make(&self, _id: i32) -> Result<Vec<CarModel>, anyhow::Error> {
            Ok(vec![])
        }
        async fn is_liked(&self, _uid: Uuid, _lid: Uuid) -> Result<bool, anyhow::Error> {
            Ok(false)
        }
        async fn is_favorited(&self, _uid: Uuid, _lid: Uuid) -> Result<bool, anyhow::Error> {
            Ok(false)
        }
        async fn toggle_like(
            &self,
            _uid: Uuid,
            _lid: Uuid,
        ) -> Result<(bool, i32), anyhow::Error> {
            Ok((true, 1))
        }
        async fn toggle_favorite(&self, _uid: Uuid, _lid: Uuid) -> Result<bool, anyhow::Error> {
            Ok(true)
        }
        async fn get_liked_ids(
            &self,
            _uid: Uuid,
            _ids: &[Uuid],
        ) -> Result<Vec<Uuid>, anyhow::Error> {
            Ok(vec![])
        }
        async fn get_favorited_ids(
            &self,
            _uid: Uuid,
            _ids: &[Uuid],
        ) -> Result<Vec<Uuid>, anyhow::Error> {
            Ok(vec![])
        }
        async fn increment_views(&self, _id: Uuid) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn create_listing(
            &self,
            _user_id: Uuid,
            _req: &CreateListingRequest,
        ) -> Result<Uuid, anyhow::Error> {
            Ok(Uuid::new_v4())
        }
        async fn update_listing(
            &self,
            _id: Uuid,
            _req: &UpdateListingRequest,
        ) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn archive_listing(&self, _id: Uuid, _reason: &str) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn is_owner(
            &self,
            _listing_id: Uuid,
            _user_id: Uuid,
        ) -> Result<bool, anyhow::Error> {
            Ok(true)
        }
        async fn add_photo(
            &self,
            _listing_id: Uuid,
            _url: String,
            _thumbnail_url: Option<String>,
            _sort_order: i16,
            _is_primary: bool,
        ) -> Result<Uuid, anyhow::Error> {
            Ok(Uuid::new_v4())
        }
        async fn delete_photo(&self, _photo_id: Uuid) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn count_photos(&self, _listing_id: Uuid) -> Result<i64, anyhow::Error> {
            Ok(0)
        }
        async fn get_photo_by_id(
            &self,
            _photo_id: Uuid,
        ) -> Result<Option<ListingPhoto>, anyhow::Error> {
            Ok(None)
        }
        async fn get_user_listings(
            &self,
            _user_id: Uuid,
            _cursor_created_at: Option<DateTime<Utc>>,
            _cursor_id: Option<Uuid>,
            _limit: i64,
        ) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(self.feed_rows.clone())
        }
        async fn get_user_favorites(
            &self,
            _user_id: Uuid,
            _cursor_created_at: Option<DateTime<Utc>>,
            _cursor_id: Option<Uuid>,
            _limit: i64,
        ) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(vec![])
        }
        async fn get_user_likes(
            &self,
            _user_id: Uuid,
            _cursor_created_at: Option<DateTime<Utc>>,
            _cursor_id: Option<Uuid>,
            _limit: i64,
        ) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(vec![])
        }
        async fn get_user_archived_listings(
            &self,
            _user_id: Uuid,
            _cursor_created_at: Option<DateTime<Utc>>,
            _cursor_id: Option<Uuid>,
            _limit: i64,
        ) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(vec![])
        }
        async fn count_user_listings(&self, _user_id: Uuid) -> Result<i64, anyhow::Error> {
            Ok(self.feed_rows.len() as i64)
        }
        async fn boost_listing(&self, _listing_id: Uuid, _user_id: Uuid) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn add_promoted_stars(&self, _listing_id: Uuid, _user_id: Uuid, stars: i32) -> Result<i32, anyhow::Error> {
            Ok(stars)
        }
        async fn get_promoted(&self, _user_id: Option<Uuid>, _limit: i64) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(vec![])
        }
        async fn extend_listing(&self, _listing_id: Uuid, _user_id: Uuid) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn feed_by_ids(&self, _ids: &[Uuid]) -> Result<Vec<FeedRow>, anyhow::Error> {
            Ok(vec![])
        }
        async fn get_listing_attrs(&self, _id: Uuid) -> Result<Option<ListingAttrsRow>, anyhow::Error> {
            Ok(None)
        }
    }

    struct MockPreferenceRepo;

    #[async_trait]
    impl PreferenceRepository for MockPreferenceRepo {
        async fn get(&self, _user_id: Uuid) -> Result<Option<crate::domain::user_preference::UserPreference>, anyhow::Error> {
            Ok(None)
        }
        async fn upsert(&self, _pref: &crate::domain::user_preference::UserPreference) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn scored_feed(&self, _user_id: Uuid, _pref: &crate::domain::user_preference::UserPreference) -> Result<Vec<Uuid>, anyhow::Error> {
            Ok(vec![])
        }
    }

    // We cannot construct a real ViewCounter without Redis, so we test service logic
    // that doesn't require it.

    #[test]
    fn test_parse_csv_some() {
        let input = Some("petrol,diesel,electric".to_string());
        let result = CarService::parse_csv(&input);
        assert_eq!(result, vec!["petrol", "diesel", "electric"]);
    }

    #[test]
    fn test_parse_csv_none() {
        let result = CarService::parse_csv(&None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_csv_empty_string() {
        let result = CarService::parse_csv(&Some("".to_string()));
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_csv_with_spaces() {
        let input = Some(" petrol , diesel ".to_string());
        let result = CarService::parse_csv(&input);
        assert_eq!(result, vec!["petrol", "diesel"]);
    }

    #[test]
    fn test_cursor_roundtrip() {
        let data = CursorData {
            c: Some("2026-03-20T12:00:00+00:00".to_string()),
            i: Uuid::new_v4().to_string(),
            p: Some(15000),
            m: Some(50000),
        };
        let encoded = CarService::encode_cursor(&data);
        let decoded = CarService::decode_cursor(&encoded).unwrap();

        assert_eq!(data.c, decoded.c);
        assert_eq!(data.i, decoded.i);
        assert_eq!(data.p, decoded.p);
        assert_eq!(data.m, decoded.m);
    }

    #[test]
    fn test_cursor_decode_invalid() {
        let result = CarService::decode_cursor("not-valid-base64!!!");
        assert!(result.is_err());
    }

    fn make_test_service(repo: MockCarRepo) -> CarService {
        let redis_pool = deadpool_redis::Config::from_url("redis://localhost")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .unwrap();
        let pref_repo: Arc<dyn PreferenceRepository> = Arc::new(MockPreferenceRepo);
        let pref_svc = Arc::new(PreferenceService::new(pref_repo.clone()));
        let feed_cache = Arc::new(FeedCache::new(redis_pool.clone()));
        CarService::new(
            Arc::new(repo),
            Arc::new(ViewCounter::new(redis_pool)),
            pref_svc,
            feed_cache,
            pref_repo,
        )
    }

    #[test]
    fn test_get_filter_options_has_all_types() {
        let service = make_test_service(MockCarRepo::new());
        let opts = service.get_filter_options();

        assert!(!opts.fuel_types.is_empty());
        assert!(!opts.body_types.is_empty());
        assert!(!opts.transmission_types.is_empty());
        assert!(!opts.drive_types.is_empty());
        assert!(!opts.features.is_empty());
        assert!(opts.fuel_types.contains(&"petrol"));
        assert!(opts.body_types.contains(&"sedan"));
        assert!(opts.transmission_types.contains(&"manual"));
        assert!(opts.drive_types.contains(&"awd"));
        assert!(opts.features.contains(&"abs"));
    }

    #[test]
    fn test_cursor_minimal_fields() {
        let data = CursorData {
            c: None,
            i: Uuid::new_v4().to_string(),
            p: None,
            m: None,
        };
        let encoded = CarService::encode_cursor(&data);
        let decoded = CarService::decode_cursor(&encoded).unwrap();
        assert!(decoded.c.is_none());
        assert!(decoded.p.is_none());
        assert!(decoded.m.is_none());
    }
}
