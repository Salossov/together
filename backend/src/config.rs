use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub backup_database_url: Option<String>,
    pub jwt_secret: String,
    pub jwt_ttl_days: i64,
    pub cors_origins: Vec<String>,

    pub ms_client_id: String,
    pub ms_client_secret: String,
    pub ms_redirect_uri: String,
    pub ms_legacy_launcher: bool,

    pub mineskin_api_key: Option<String>,
    pub manifest_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        fn req(k: &str) -> anyhow::Result<String> {
            env::var(k).map_err(|_| anyhow::anyhow!("missing env var {k}"))
        }
        fn opt(k: &str) -> Option<String> {
            env::var(k).ok().filter(|s| !s.is_empty())
        }

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: req("DATABASE_URL")?,
            backup_database_url: opt("BACKUP_DATABASE_URL"),
            jwt_secret: req("JWT_SECRET")?,
            jwt_ttl_days: env::var("JWT_TTL_DAYS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            cors_origins,

            ms_client_id: opt("MS_CLIENT_ID").unwrap_or_default(),
            ms_client_secret: opt("MS_CLIENT_SECRET").unwrap_or_default(),
            ms_redirect_uri: opt("MS_REDIRECT_URI").unwrap_or_default(),
            ms_legacy_launcher: env::var("MS_LEGACY_LAUNCHER")
                .ok()
                .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(false),

            mineskin_api_key: opt("MINESKIN_API_KEY"),
            manifest_path: env::var("MANIFEST_PATH")
                .unwrap_or_else(|_| "./data/manifest.json".into()),
        })
    }
}
