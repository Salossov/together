use std::sync::Arc;

use axum::{extract::State, Json};

use crate::{
    auth::jwt,
    db::P,
    error::{AppError, AppResult},
    models::{HandshakeReq, HandshakeResp, SkinPayload, User},
    state::AppState,
};

/// Вызывается серверным модом во время фазы LOGIN handshake.
pub async fn validate_handshake(
    State(state): State<Arc<AppState>>,
    Json(req): Json<HandshakeReq>,
) -> AppResult<Json<HandshakeResp>> {
    let claims = jwt::verify(&state, &req.jwt_token)?;

    if !claims.username.eq_ignore_ascii_case(&req.username) {
        return Err(AppError::Unauthorized);
    }

    let user: Option<User> = sqlx::query_as(&state.db.sql(
        "SELECT id, username, password_hash, email, internal_uuid,
                skin_base64, skin_signature, auth_type
         FROM users WHERE internal_uuid = ? LIMIT 1",
    ))
    .bind(&claims.sub)
    .fetch_optional(&state.db.primary)
    .await?;
    let user = user.ok_or(AppError::Unauthorized)?;

    if !user.username.eq_ignore_ascii_case(&req.username) {
        state
            .db
            .write(
                "UPDATE users SET username = ? WHERE id = ?",
                vec![P::from(&req.username), P::from(user.id)],
            )
            .await?;
    }

    let skin = match (user.skin_base64, user.skin_signature) {
        (Some(value), Some(signature)) => Some(SkinPayload { value, signature }),
        _ => None,
    };

    Ok(Json(HandshakeResp {
        status: "allowed",
        internal_uuid: user.internal_uuid,
        skin,
    }))
}
