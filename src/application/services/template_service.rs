use std::collections::HashMap;
use std::sync::Arc;
use crate::application::ports::template_repository::TemplateRepository;

pub struct TemplateService {
    repo: Arc<dyn TemplateRepository>,
    redis_pool: deadpool_redis::Pool,
}

impl TemplateService {
    pub fn new(repo: Arc<dyn TemplateRepository>, redis_pool: deadpool_redis::Pool) -> Self {
        Self { repo, redis_pool }
    }

    pub async fn get(&self, key: &str) -> Result<String, anyhow::Error> {
        // Try Redis first
        if let Ok(mut conn) = self.redis_pool.get().await {
            let cache_key = format!("tmpl:{}", key);
            if let Ok(val) = deadpool_redis::redis::cmd("GET")
                .arg(&cache_key)
                .query_async::<String>(&mut conn)
                .await
            {
                return Ok(val);
            }
        }

        // Fallback to DB
        let tmpl = self
            .repo
            .get(key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", key))?;

        // Cache in Redis (5 min TTL)
        if let Ok(mut conn) = self.redis_pool.get().await {
            let cache_key = format!("tmpl:{}", key);
            let _ = deadpool_redis::redis::cmd("SETEX")
                .arg(&cache_key)
                .arg(300)
                .arg(&tmpl.body)
                .query_async::<()>(&mut conn)
                .await;
        }

        Ok(tmpl.body)
    }

    pub async fn render(
        &self,
        key: &str,
        params: &HashMap<&str, String>,
    ) -> Result<String, anyhow::Error> {
        let mut body = self.get(key).await?;
        for (k, v) in params {
            body = body.replace(&format!("{{{}}}", k), v);
        }
        Ok(body)
    }
}
