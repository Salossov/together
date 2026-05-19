#![deny(unsafe_code)]

mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod state;

use std::sync::Arc;

use anyhow::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // .env (молча игнорируем отсутствие)
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let cfg = config::Config::from_env().context("loading config")?;
    tracing::info!("starting together-auth");

    let db = db::init(
        &cfg.database_url,
        cfg.backup_database_url.as_deref(),
    )
    .await?;

    let http = reqwest::Client::builder()
        .user_agent("together-auth/0.1")
        .build()?;

    let state = Arc::new(state::AppState {
        db,
        http,
        cfg: cfg.clone(),
        jwt_enc: jsonwebtoken::EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        jwt_dec: jsonwebtoken::DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    });

    let app = routes::router(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr)
        .await
        .with_context(|| format!("bind {}", cfg.bind_addr))?;
    tracing::info!("listening on http://{}", cfg.bind_addr);

    axum::serve(listener, app).await?;
    Ok(())
}
