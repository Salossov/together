use std::sync::Arc;

use axum::{extract::State, Json};

use crate::{
    auth::jwt,
    error::{AppError, AppResult},
    models::{Manifest, VerifyTokenReq, VerifyTokenResp, VersionCheckReq, VersionCheckResp},
    state::AppState,
};

pub async fn verify_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyTokenReq>,
) -> Json<VerifyTokenResp> {
    match jwt::verify(&state, &req.token) {
        Ok(c) => Json(VerifyTokenResp {
            status: "valid",
            username: Some(c.username),
            internal_uuid: Some(c.sub),
        }),
        Err(_) => Json(VerifyTokenResp {
            status: "invalid",
            username: None,
            internal_uuid: None,
        }),
    }
}

pub async fn version_check(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VersionCheckReq>,
) -> AppResult<Json<VersionCheckResp>> {
    let raw = tokio::fs::read(&state.cfg.manifest_path).await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!(
            "read manifest {}: {e}",
            state.cfg.manifest_path
        ))
    })?;
    let manifest: Manifest = serde_json::from_slice(&raw)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("parse manifest: {e}")))?;

    let mut update = Vec::new();
    for f in &manifest.files {
        let client_hash = req.files.get(&f.path);
        if client_hash.map(|h| h.eq_ignore_ascii_case(&f.sha256)) != Some(true) {
            update.push(f.clone());
        }
    }

    let server_paths: std::collections::HashSet<&String> =
        manifest.files.iter().map(|f| &f.path).collect();
    let remove: Vec<String> = req
        .files
        .keys()
        .filter(|p| !server_paths.contains(p))
        .cloned()
        .collect();

    Ok(Json(VersionCheckResp { update, remove }))
}
