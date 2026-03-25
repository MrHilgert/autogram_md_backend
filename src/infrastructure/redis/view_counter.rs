use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;

pub struct ViewCounter {
    redis: RedisPool,
}

impl ViewCounter {
    pub fn new(redis: RedisPool) -> Self {
        Self { redis }
    }

    /// Returns true if this is a new view (not seen in last hour).
    /// Uses Redis SET NX EX for deduplication.
    pub async fn record_view(&self, user_id: &str, car_id: &str) -> Result<bool, anyhow::Error> {
        let mut conn = self.redis.get().await?;
        let dedup_key = format!("cars:viewed:{}:{}", user_id, car_id);

        // SET key "1" NX EX 3600 — only sets if not exists, expires in 1 hour
        let is_new: bool = redis::cmd("SET")
            .arg(&dedup_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(3600i64)
            .query_async::<bool>(&mut conn)
            .await
            .unwrap_or(false);

        if is_new {
            let counter_key = format!("cars:views:{}", car_id);
            let _: () = conn.incr(&counter_key, 1i64).await?;
        }

        Ok(is_new)
    }
}
