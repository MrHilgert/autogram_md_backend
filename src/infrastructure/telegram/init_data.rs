use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub language_code: Option<String>,
    pub is_premium: Option<bool>,
}

/// Validates Telegram WebApp initData using HMAC-SHA256.
///
/// Returns the parsed `TelegramUser` on success, or an error string
/// if the hash is invalid, data is expired, or the user payload is malformed.
pub fn validate_init_data(init_data: &str, bot_token: &str) -> Result<TelegramUser, String> {
    // Parse the query string
    let params: BTreeMap<String, String> = url::form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();

    // Extract hash
    let hash = params.get("hash").ok_or("Missing hash parameter")?;

    // Build data check string (all params except hash, sorted alphabetically)
    let data_check_string: String = params
        .iter()
        .filter(|(k, _)| k.as_str() != "hash")
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    // Compute secret key: HMAC-SHA256("WebAppData", bot_token)
    let mut secret_mac =
        HmacSha256::new_from_slice(b"WebAppData").map_err(|e| format!("HMAC error: {}", e))?;
    secret_mac.update(bot_token.as_bytes());
    let secret_key = secret_mac.finalize().into_bytes();

    // Compute hash: HMAC-SHA256(secret_key, data_check_string)
    let mut mac =
        HmacSha256::new_from_slice(&secret_key).map_err(|e| format!("HMAC error: {}", e))?;
    mac.update(data_check_string.as_bytes());
    // Constant-time comparison
    let hash_bytes = hex::decode(hash).map_err(|_| "Invalid hash format".to_string())?;
    mac.verify_slice(&hash_bytes).map_err(|_| "Invalid hash".to_string())?;

    // Check auth_date is not too old (allow 24 hours)
    if let Some(auth_date_str) = params.get("auth_date") {
        let auth_date: i64 = auth_date_str.parse().map_err(|_| "Invalid auth_date")?;
        let now = chrono::Utc::now().timestamp();
        if now - auth_date > 86400 {
            return Err("Init data expired".to_string());
        }
    }

    // Parse user data
    let user_json = params.get("user").ok_or("Missing user parameter")?;
    let user: TelegramUser =
        serde_json::from_str(user_json).map_err(|e| format!("Failed to parse user data: {}", e))?;

    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_valid_init_data(bot_token: &str) -> String {
        use std::fmt::Write;

        let user_json = r#"{"id":123456,"first_name":"Test","last_name":"User","username":"testuser"}"#;
        let auth_date = chrono::Utc::now().timestamp().to_string();

        // Build data check string (alphabetically sorted keys, excluding hash)
        let mut params = BTreeMap::new();
        params.insert("auth_date".to_string(), auth_date.clone());
        params.insert("user".to_string(), user_json.to_string());

        let data_check_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        // Compute secret key
        let mut secret_mac = HmacSha256::new_from_slice(b"WebAppData").unwrap();
        secret_mac.update(bot_token.as_bytes());
        let secret_key = secret_mac.finalize().into_bytes();

        // Compute hash
        let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
        mac.update(data_check_string.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());

        // Build query string
        let mut result = String::new();
        write!(
            result,
            "auth_date={}&user={}&hash={}",
            auth_date,
            url::form_urlencoded::byte_serialize(user_json.as_bytes()).collect::<String>(),
            hash
        )
        .unwrap();

        result
    }

    #[test]
    fn test_validate_init_data_valid() {
        let bot_token = "7777777777:AAFtest-bot-token-for-testing";
        let init_data = build_valid_init_data(bot_token);

        let result = validate_init_data(&init_data, bot_token);
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.id, 123456);
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.username.as_deref(), Some("testuser"));
    }

    #[test]
    fn test_validate_init_data_invalid_hash() {
        let init_data = "user=%7B%22id%22%3A123%2C%22first_name%22%3A%22Test%22%7D&auth_date=9999999999&hash=invalidhash";
        let result = validate_init_data(init_data, "test_bot_token");
        assert!(result.is_err());
        // "invalidhash" is not valid hex, so we get "Invalid hash format"
        assert_eq!(result.unwrap_err(), "Invalid hash format");
    }

    #[test]
    fn test_validate_init_data_wrong_hash() {
        // Valid hex but wrong HMAC value
        let init_data = "user=%7B%22id%22%3A123%2C%22first_name%22%3A%22Test%22%7D&auth_date=9999999999&hash=aabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccdd";
        let result = validate_init_data(init_data, "test_bot_token");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid hash");
    }

    #[test]
    fn test_validate_init_data_missing_hash() {
        let init_data = "user=%7B%22id%22%3A123%7D&auth_date=9999999999";
        let result = validate_init_data(init_data, "test_bot_token");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing hash parameter");
    }

    #[test]
    fn test_validate_init_data_missing_user() {
        // Build valid hash but without user param
        let bot_token = "test_token";
        let auth_date = chrono::Utc::now().timestamp().to_string();

        let data_check_string = format!("auth_date={}", auth_date);

        let mut secret_mac = HmacSha256::new_from_slice(b"WebAppData").unwrap();
        secret_mac.update(bot_token.as_bytes());
        let secret_key = secret_mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
        mac.update(data_check_string.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());

        let init_data = format!("auth_date={}&hash={}", auth_date, hash);
        let result = validate_init_data(&init_data, bot_token);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing user parameter");
    }

    #[test]
    fn test_validate_init_data_expired() {
        // Use a very old auth_date with a valid hash
        let bot_token = "test_token";
        let user_json = r#"{"id":123,"first_name":"Test"}"#;
        let auth_date = "1000000000"; // ~2001, definitely expired

        let mut params = BTreeMap::new();
        params.insert("auth_date".to_string(), auth_date.to_string());
        params.insert("user".to_string(), user_json.to_string());

        let data_check_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let mut secret_mac = HmacSha256::new_from_slice(b"WebAppData").unwrap();
        secret_mac.update(bot_token.as_bytes());
        let secret_key = secret_mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
        mac.update(data_check_string.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());

        let init_data = format!(
            "auth_date={}&user={}&hash={}",
            auth_date,
            url::form_urlencoded::byte_serialize(user_json.as_bytes()).collect::<String>(),
            hash
        );

        let result = validate_init_data(&init_data, bot_token);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Init data expired");
    }
}
