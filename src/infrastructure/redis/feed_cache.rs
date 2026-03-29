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
        let capped = if listing_ids.len() > 500 {
            &listing_ids[..500]
        } else {
            listing_ids
        };
        let value = serde_json::to_string(capped)?;
        conn.set_ex::<_, _, ()>(&key, &value, FEED_TTL).await?;
        Ok(())
    }

    /// Delete the personalized feed cache for a specific user.
    pub async fn invalidate_user_feed(&self, user_id: Uuid) -> Result<(), anyhow::Error> {
        let key = format!("feed:personal:{}", user_id);
        let mut conn = self.redis.get().await?;
        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
        Ok(())
    }

    /// Delete all personalized feed caches (e.g. after new listing created).
    pub async fn invalidate_all_feeds(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.redis.get().await?;
        let mut cursor: u64 = 0;
        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg("feed:personal:*")
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;
            if !keys.is_empty() {
                let _: () = redis::cmd("DEL").arg(&keys).query_async(&mut conn).await?;
            }
            if next_cursor == 0 {
                break;
            }
            cursor = next_cursor;
        }
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
