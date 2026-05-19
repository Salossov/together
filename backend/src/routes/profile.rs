use std::sync::Arc;

use axum::{extract::{Multipart, State}, Json};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::json;

use crate::{
    auth::extractor::AuthUser,
    db::P,
    error::{AppError, AppResult},
    models::User,
    state::AppState,
};

pub async fn me(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> AppResult<Json<User>> {
    let user: Option<User> = sqlx::query_as(&state.db.sql(
        "SELECT id, username, password_hash, email, internal_uuid,
                skin_base64, skin_signature, auth_type
         FROM users WHERE internal_uuid = ? LIMIT 1",
    ))
    .bind(&claims.sub)
    .fetch_optional(&state.db.primary)
    .await?;
    user.map(Json).ok_or(AppError::NotFound)
}

/// Загрузка скина: multipart/form-data, поле "file" с PNG (64x64 или 64x32).
pub async fn upload_skin(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    mut mp: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart: {e}")))?
    {
        if field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("read field: {e}")))?;
            bytes = Some(data.to_vec());
            break;
        }
    }
    let bytes = bytes.ok_or_else(|| AppError::BadRequest("missing 'file' field".into()))?;

    if bytes.len() > 256 * 1024 {
        return Err(AppError::BadRequest("skin too large (>256KiB)".into()));
    }

    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .map_err(|e| AppError::BadRequest(format!("not a PNG: {e}")))?;
    let (w, h) = (img.width(), img.height());
    if !(w == 64 && (h == 64 || h == 32)) {
        return Err(AppError::BadRequest(format!(
            "skin must be 64x64 or 64x32, got {w}x{h}"
        )));
    }

    let b64 = B64.encode(&bytes);

    let signature = if let Some(key) = &state.cfg.mineskin_api_key {
        try_mineskin_sign(&state.http, key, &bytes).await.ok().flatten()
    } else {
        None
    };

    state
        .db
        .write(
            "UPDATE users SET skin_base64 = ?, skin_signature = ? WHERE internal_uuid = ?",
            vec![
                P::from(b64.clone()),
                P::from(signature.clone()),
                P::from(&claims.sub),
            ],
        )
        .await?;

    Ok(Json(json!({
        "ok": true,
        "signed": signature.is_some(),
        "size": bytes.len(),
    })))
}

async fn try_mineskin_sign(
    http: &reqwest::Client,
    api_key: &str,
    png: &[u8],
) -> anyhow::Result<Option<String>> {
    let part = reqwest::multipart::Part::bytes(png.to_vec())
        .file_name("skin.png")
        .mime_str("image/png")?;
    let form = reqwest::multipart::Form::new().part("file", part);

    #[derive(serde::Deserialize)]
    struct MS { data: MSData }
    #[derive(serde::Deserialize)]
    struct MSData { texture: MSTex }
    #[derive(serde::Deserialize)]
    struct MSTex { signature: Option<String> }

    let r: MS = http
        .post("https://api.mineskin.org/generate/upload")
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(r.data.texture.signature)
}
