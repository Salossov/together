use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub internal_uuid: String,
    pub skin_base64: Option<String>,
    pub skin_signature: Option<String>,
    pub auth_type: String,
}

// ----- DTO -----

#[derive(Debug, Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResp {
    pub token: String,
    pub username: String,
    pub internal_uuid: String,
    pub auth_type: String,
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftReq {
    /// authorization_code, полученный фронтом из Microsoft OAuth redirect
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTokenReq {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyTokenResp {
    pub status: &'static str, // "valid" | "invalid"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionCheckReq {
    /// `{"path/to/mod.jar": "sha256hex"}`
    pub files: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct VersionCheckResp {
    pub update: Vec<UpdateFile>,
    pub remove: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateFile {
    pub path: String,
    pub sha256: String,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<UpdateFile>,
}

// ----- handshake -----

#[derive(Debug, Deserialize)]
pub struct HandshakeReq {
    pub username: String,
    pub jwt_token: String,
}

#[derive(Debug, Serialize)]
pub struct SkinPayload {
    pub value: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct HandshakeResp {
    pub status: &'static str, // "allowed"
    pub internal_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<SkinPayload>,
}
