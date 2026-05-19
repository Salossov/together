use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // internal_uuid
    pub username: String,
    pub auth_type: String,     // "LOCAL" | "MICROSOFT"
    pub iat: i64,
    pub exp: i64,
}

pub fn issue(state: &AppState, internal_uuid: &str, username: &str, auth_type: &str)
    -> Result<String, AppError>
{
    let now = Utc::now();
    let exp = now + Duration::days(state.cfg.jwt_ttl_days);
    let claims = Claims {
        sub: internal_uuid.to_string(),
        username: username.to_string(),
        auth_type: auth_type.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };
    encode(&Header::new(Algorithm::HS256), &claims, &state.jwt_enc)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("jwt encode: {e}")))
}

pub fn verify(state: &AppState, token: &str) -> Result<Claims, AppError> {
    let mut v = Validation::new(Algorithm::HS256);
    v.leeway = 30;
    decode::<Claims>(token, &state.jwt_dec, &v)
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
}
