use crate::{config::Config, db::Db};
use jsonwebtoken::{DecodingKey, EncodingKey};

pub struct AppState {
    pub db: Db,
    pub http: reqwest::Client,
    pub cfg: Config,
    pub jwt_enc: EncodingKey,
    pub jwt_dec: DecodingKey,
}
