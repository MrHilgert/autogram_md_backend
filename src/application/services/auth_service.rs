use std::sync::Arc;

use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::ports::user_repository::UserRepository;
use crate::domain::user::NewUser;
use crate::infrastructure::telegram::init_data::validate_init_data;

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User UUID
    pub sub: String,
    pub telegram_id: i64,
    pub exp: i64,
    pub iat: i64,
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    bot_token: String,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        bot_token: String,
        jwt_secret: String,
    ) -> Self {
        Self {
            user_repo,
            bot_token,
            jwt_secret,
        }
    }

    /// Authenticates a Telegram WebApp user via initData.
    ///
    /// Validates the HMAC signature, upserts the user in the database,
    /// and returns a JWT token along with the user entity.
    pub async fn authenticate(
        &self,
        init_data: &str,
    ) -> Result<(String, crate::domain::user::User), anyhow::Error> {
        // Validate Telegram initData
        let tg_user = validate_init_data(init_data, &self.bot_token)
            .map_err(|e| anyhow::anyhow!("Auth failed: {}", e))?;

        // Upsert user
        let new_user = NewUser {
            telegram_id: tg_user.id,
            username: tg_user.username,
            first_name: tg_user.first_name,
            last_name: tg_user.last_name,
            avatar_url: tg_user.photo_url,
        };
        let user = self
            .user_repo
            .upsert(&new_user)
            .await
            .context("Failed to upsert user")?;

        // Generate JWT
        let token = self.generate_jwt(&user.id, user.telegram_id)?;

        Ok((token, user))
    }

    /// Generates a JWT token valid for 7 days.
    pub fn generate_jwt(&self, user_id: &Uuid, telegram_id: i64) -> Result<String, anyhow::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            telegram_id,
            iat: now.timestamp(),
            exp: (now + Duration::days(7)).timestamp(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;
        Ok(token)
    }

    /// Verifies and decodes a JWT token, returning the claims.
    pub fn verify_jwt(&self, token: &str) -> Result<Claims, anyhow::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct DummyRepo;

    #[async_trait]
    impl UserRepository for DummyRepo {
        async fn find_by_telegram_id(
            &self,
            _: i64,
        ) -> Result<Option<crate::domain::user::User>, anyhow::Error> {
            Ok(None)
        }
        async fn find_by_id(
            &self,
            _: Uuid,
        ) -> Result<Option<crate::domain::user::User>, anyhow::Error> {
            Ok(None)
        }
        async fn upsert(
            &self,
            _: &NewUser,
        ) -> Result<crate::domain::user::User, anyhow::Error> {
            unimplemented!()
        }
        async fn update_last_active(&self, _: Uuid) -> Result<(), anyhow::Error> {
            Ok(())
        }
    }

    fn make_service() -> AuthService {
        AuthService::new(
            Arc::new(DummyRepo),
            "test_bot_token".to_string(),
            "test_jwt_secret_that_is_long_enough".to_string(),
        )
    }

    #[test]
    fn test_jwt_roundtrip() {
        let service = make_service();
        let user_id = Uuid::new_v4();
        let telegram_id = 12345i64;

        let token = service.generate_jwt(&user_id, telegram_id).unwrap();
        let claims = service.verify_jwt(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.telegram_id, telegram_id);
    }

    #[test]
    fn test_jwt_invalid_token() {
        let service = make_service();
        let result = service.verify_jwt("this.is.invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_wrong_secret() {
        let service = make_service();
        let user_id = Uuid::new_v4();
        let token = service.generate_jwt(&user_id, 123).unwrap();

        // Verify with a different secret
        let other_service = AuthService::new(
            Arc::new(DummyRepo),
            "test_bot_token".to_string(),
            "completely_different_secret_key".to_string(),
        );
        let result = other_service.verify_jwt(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_claims_expiration() {
        let service = make_service();
        let user_id = Uuid::new_v4();

        let token = service.generate_jwt(&user_id, 999).unwrap();
        let claims = service.verify_jwt(&token).unwrap();

        // Token should expire in ~7 days
        let now = Utc::now().timestamp();
        let seven_days = 7 * 24 * 3600;
        assert!(claims.exp - now >= seven_days - 5); // allow 5s tolerance
        assert!(claims.exp - now <= seven_days + 5);
    }
}
