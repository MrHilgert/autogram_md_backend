use aws_sdk_s3::config::{BehaviorVersion, Builder, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use uuid::Uuid;

pub struct PhotoStorage {
    client: S3Client,
    bucket: String,
    public_url: String,
}

impl PhotoStorage {
    pub async fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        region: &str,
        bucket: &str,
        public_url: &str,
    ) -> Self {
        let creds = Credentials::new(access_key, secret_key, None, None, "env");
        let config = Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new(region.to_string()))
            .credentials_provider(creds)
            .force_path_style(true)
            .build();
        let client = S3Client::from_conf(config);
        Self {
            client,
            bucket: bucket.to_string(),
            public_url: public_url.trim_end_matches('/').to_string(),
        }
    }

    /// Upload bytes to S3/R2 and return the public CDN URL.
    pub async fn upload(
        &self,
        listing_id: Uuid,
        data: Vec<u8>,
        content_type: &str,
        suffix: &str,
    ) -> Result<String, anyhow::Error> {
        let file_id = Uuid::new_v4();
        let key = format!("listings/{}/{}{}", listing_id, file_id, suffix);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await?;

        Ok(format!("{}/{}", self.public_url, key))
    }

    /// Delete an object by its public URL.
    pub async fn delete(&self, url: &str) -> Result<(), anyhow::Error> {
        // Extract key from full CDN URL or legacy /photos/ path
        let key = if let Some(k) = url.strip_prefix(&format!("{}/", self.public_url)) {
            k
        } else if let Some(k) = url.strip_prefix("/photos/") {
            k
        } else {
            return Ok(());
        };

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }
}
