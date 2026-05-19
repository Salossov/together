use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::{jwt, microsoft::{self, LAUNCHER_CLIENT_ID, LAUNCHER_REDIRECT_URI, LAUNCHER_SCOPE}, password},
    db::P,
    error::{AppError, AppResult},
    models::{AuthResp, LoginReq, MicrosoftReq, RegisterReq, User},
    state::AppState,
};

const USERNAME_RE: &str = r"^[A-Za-z0-9_]{3,16}$";

fn validate_username(name: &str) -> Result<(), AppError> {
    let ok = name.len() >= 3
        && name.len() <= 16
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if ok {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "username must match {USERNAME_RE}"
        )))
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterReq>,
) -> AppResult<Json<AuthResp>> {
    validate_username(&req.username)?;
    if req.password.len() < 6 {
        return Err(AppError::BadRequest("password too short (min 6)".into()));
    }

    // Уникальность ника / email
    let exists: Option<(i64,)> = sqlx::query_as(&state.db.sql(
        "SELECT id FROM users WHERE username = ? LIMIT 1",
    ))
    .bind(&req.username)
    .fetch_optional(&state.db.primary)
    .await?;
    if exists.is_some() {
        return Err(AppError::Conflict("username taken".into()));
    }
    if let Some(email) = &req.email {
        let e: Option<(i64,)> = sqlx::query_as(&state.db.sql(
            "SELECT id FROM users WHERE email = ? LIMIT 1",
        ))
        .bind(email)
        .fetch_optional(&state.db.primary)
        .await?;
        if e.is_some() {
            return Err(AppError::Conflict("email taken".into()));
        }
    }

    let hash = password::hash(req.password).await?;
    let internal_uuid = Uuid::new_v4().to_string();

    state
        .db
        .write(
            "INSERT INTO users (username, password_hash, email, internal_uuid, auth_type)
             VALUES (?, ?, ?, ?, 'LOCAL')",
            vec![
                P::from(&req.username),
                P::from(hash),
                P::from(req.email.clone()),
                P::from(&internal_uuid),
            ],
        )
        .await?;

    let token = jwt::issue(&state, &internal_uuid, &req.username, "LOCAL")?;

    Ok(Json(AuthResp {
        token,
        username: req.username,
        internal_uuid,
        auth_type: "LOCAL".into(),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginReq>,
) -> AppResult<Json<AuthResp>> {
    let user: Option<User> = sqlx::query_as(&state.db.sql(
        "SELECT id, username, password_hash, email, internal_uuid,
                skin_base64, skin_signature, auth_type
         FROM users WHERE username = ? LIMIT 1",
    ))
    .bind(&req.username)
    .fetch_optional(&state.db.primary)
    .await?;

    let user = user.ok_or(AppError::Unauthorized)?;
    if user.auth_type != "LOCAL" {
        return Err(AppError::BadRequest(
            "this account uses Microsoft sign-in".into(),
        ));
    }

    if !password::verify(req.password, user.password_hash.clone()).await? {
        return Err(AppError::Unauthorized);
    }

    let token = jwt::issue(&state, &user.internal_uuid, &user.username, &user.auth_type)?;
    Ok(Json(AuthResp {
        token,
        username: user.username,
        internal_uuid: user.internal_uuid,
        auth_type: user.auth_type,
    }))
}

/// Возвращает authorize URL и режим:
///  - mode="redirect"     — обычный OAuth, фронт делает window.location=url
///  - mode="manual_paste" — launcher-режим, фронт открывает new tab,
///                          юзер копирует URL обратно в форму
pub async fn microsoft_url(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    if state.cfg.ms_legacy_launcher {
        let scope = LAUNCHER_SCOPE;
        let url = format!(
            "https://login.live.com/oauth20_authorize.srf\
             ?client_id={cid}\
             &response_type=code\
             &redirect_uri={redir}\
             &scope={scope}\
             &prompt=select_account",
            cid   = urlencoding(LAUNCHER_CLIENT_ID),
            redir = urlencoding(LAUNCHER_REDIRECT_URI),
            scope = urlencoding(scope),
        );
        return Ok(Json(json!({
            "url": url,
            "configured": true,
            "mode": "manual_paste",
            "redirect_uri": LAUNCHER_REDIRECT_URI,
        })));
    }

    if state.cfg.ms_client_id.is_empty() || state.cfg.ms_redirect_uri.is_empty() {
        return Err(AppError::BadRequest(
            "Microsoft OAuth не сконфигурирован на бэке (MS_CLIENT_ID / MS_REDIRECT_URI)".into(),
        ));
    }
    let scope = "XboxLive.signin offline_access openid email";
    let url = format!(
        "https://login.live.com/oauth20_authorize.srf\
         ?client_id={cid}\
         &response_type=code\
         &redirect_uri={redir}\
         &scope={scope}\
         &prompt=select_account",
        cid   = urlencoding(&state.cfg.ms_client_id),
        redir = urlencoding(&state.cfg.ms_redirect_uri),
        scope = urlencoding(scope),
    );
    Ok(Json(json!({
        "url": url,
        "configured": true,
        "mode": "redirect",
    })))
}

/// Минимальный URL-encoder без зависимости (нам нужно совсем чуть-чуть).
fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub async fn microsoft(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MicrosoftReq>,
) -> AppResult<Json<AuthResp>> {
    let mp = microsoft::exchange_code(&state, &req.code).await?;

    let canonical = canonical_uuid(&mp.uuid)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("bad uuid from MS")))?;

    let existing: Option<User> = sqlx::query_as(&state.db.sql(
        "SELECT id, username, password_hash, email, internal_uuid,
                skin_base64, skin_signature, auth_type
         FROM users WHERE internal_uuid = ? LIMIT 1",
    ))
    .bind(&canonical)
    .fetch_optional(&state.db.primary)
    .await?;

    match existing {
        Some(u) => {
            state
                .db
                .write(
                    "UPDATE users
                     SET username = ?, skin_base64 = ?, skin_signature = ?
                     WHERE id = ?",
                    vec![
                        P::from(&mp.username),
                        P::from(mp.skin_value.clone()),
                        P::from(mp.skin_signature.clone()),
                        P::from(u.id),
                    ],
                )
                .await?;
        }
        None => {
            // Пароль для MS-аккаунта не используется — храним случайный
            // невостанавливаемый хэш-плейсхолдер.
            let placeholder = password::hash(Uuid::new_v4().to_string()).await?;
            state
                .db
                .write(
                    "INSERT INTO users
                       (username, password_hash, email, internal_uuid,
                        skin_base64, skin_signature, auth_type)
                     VALUES (?, ?, NULL, ?, ?, ?, 'MICROSOFT')",
                    vec![
                        P::from(&mp.username),
                        P::from(placeholder),
                        P::from(&canonical),
                        P::from(mp.skin_value.clone()),
                        P::from(mp.skin_signature.clone()),
                    ],
                )
                .await?;
        }
    };

    let token = jwt::issue(&state, &canonical, &mp.username, "MICROSOFT")?;
    Ok(Json(AuthResp {
        token,
        username: mp.username,
        internal_uuid: canonical,
        auth_type: "MICROSOFT".into(),
    }))
}

fn canonical_uuid(s: &str) -> Option<String> {
    let s = s.trim();
    if s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(format!(
            "{}-{}-{}-{}-{}",
            &s[0..8],
            &s[8..12],
            &s[12..16],
            &s[16..20],
            &s[20..32]
        ))
    } else if Uuid::parse_str(s).is_ok() {
        Some(s.to_string())
    } else {
        None
    }
}
