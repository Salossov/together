use crate::error::AppError;

pub async fn hash(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST)
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("join: {e}")))?
    .map_err(|e| AppError::Internal(anyhow::anyhow!("bcrypt: {e}")))
}

pub async fn verify(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || bcrypt::verify(password.as_bytes(), &hash))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("join: {e}")))?
        .map_err(|e| AppError::Internal(anyhow::anyhow!("bcrypt: {e}")))
}
