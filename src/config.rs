use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub bot_token: String,
    pub jwt_secret: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    #[serde(default = "default_s3_region")]
    pub s3_region: String,
    #[serde(default = "default_s3_public_url")]
    pub s3_public_url: String,
    #[serde(default = "default_webapp_url")]
    pub webapp_url: String,
    #[serde(default = "default_bot_webapp_url")]
    pub bot_webapp_url: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_s3_public_url() -> String {
    "https://cdn.car.hilgert.cc".to_string()
}

fn default_webapp_url() -> String {
    "https://car.hilgert.cc".to_string()
}

fn default_bot_webapp_url() -> String {
    "https://t.me/pmrcar_bot/market".to_string()
}

impl AppConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }
}
