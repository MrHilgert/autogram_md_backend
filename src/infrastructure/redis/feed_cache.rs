use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use uuid::Uuid;

const FEED_TTL: u64 = 1800; // 30 minutes

pub struct FeedCache {
    redis: RedisPool,
}

impl FeedCache {
    pub fn new(redis: RedisPool) -> Self {
        Self { redis }
    }

    /// Cache ordered listing IDs for a user's personalized feed.
    pub async fn set_feed_snapshot(
        &self,
        user_id: Uuid,
        listing_ids: &[Uuid],
    ) -> Result<(), anyhow::Error> {
        let key = format!("feed:personal:{}", user_id);
        let mut conn = self.redis.get().await?;
        let value = serde_json::to_string(listing_ids)?;
        conn.set_ex::<_, _, ()>(&key, &value, FEED_TTL).await?;
        Ok(())
    }

    /// Get a page of listing IDs from the cached snapshot.
    /// Returns None if cache expired/missing, otherwise (page_ids, has_more).
    pub async fn get_feed_page(
        &self,
        user_id: Uuid,
        offset: usize,
        limit: usize,
    ) -> Result<Option<(Vec<Uuid>, bool)>, anyhow::Error> {
        let key = format!("feed:personal:{}", user_id);
        let mut conn = self.redis.get().await?;
        let value: Option<String> = conn.get(&key).await?;
        match value {
            None => Ok(None),
            Some(json) => {
                let all_ids: Vec<Uuid> = serde_json::from_str(&json)?;
                let page: Vec<Uuid> = all_ids
                    .iter()
                    .skip(offset)
                    .take(limit + 1)
                    .copied()
                    .collect();
                let has_more = page.len() > limit;
                let page = if has_more {
                    page[..limit].to_vec()
                } else {
                    page
                };
                Ok(Some((page, has_more)))
            }
        }
    }
}
