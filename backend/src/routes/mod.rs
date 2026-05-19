pub mod auth;
pub mod launcher;
pub mod mc_server;
pub mod profile;

use std::{sync::Arc, time::Duration};

use axum::{routing::{get, post}, Router};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
    limit::RequestBodyLimitLayer,
};

use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    let cors = build_cors(&state.cfg.cors_origins);

    Router::new()
        .route("/health", get(|| async { "ok" }))

        // фронт / панель
        .route("/api/auth/register",  post(auth::register))
        .route("/api/auth/login",     post(auth::login))
        .route("/api/auth/microsoft",      post(auth::microsoft))
        .route("/api/auth/microsoft/url",  get (auth::microsoft_url))

        // профиль игрока (защищено JWT)
        .route("/api/profile/skin",   post(profile::upload_skin))
        .route("/api/profile/me",     get (profile::me))

        // лаунчер / клиентский мод
        .route("/api/launcher/verify_token",  post(launcher::verify_token))
        .route("/api/launcher/version_check", get (launcher::version_check))

        // серверный мод (LOGIN handshake)
        .route("/api/mc-server/validate_handshake", post(mc_server::validate_handshake))

        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024)) // 2 MiB (скины ~ 6KB)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(20),
        ))
        .with_state(state)
}

fn build_cors(origins: &[String]) -> CorsLayer {
    use axum::http::{HeaderName, Method};

    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
        ])
        .max_age(Duration::from_secs(600));

    if origins.iter().any(|o| o == "*") {
        return base.allow_origin(AllowOrigin::any());
    }
    let parsed = origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect::<Vec<_>>();
    base.allow_origin(parsed).allow_credentials(true)
}
