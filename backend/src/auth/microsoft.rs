//! Microsoft OAuth → Xbox Live → XSTS → Minecraft Services
//!
//! Полная цепочка проверки лицензии Minecraft:
//!   1) code        -> MS access_token  (login.live.com/oauth20_token.srf)
//!   2) MS token    -> XBL token        (user.auth.xboxlive.com/user/authenticate)
//!   3) XBL         -> XSTS             (xsts.auth.xboxlive.com/xsts/authorize)
//!   4) XSTS        -> MC access_token  (api.minecraftservices.com/authentication/login_with_xbox)
//!   5) MC token    -> profile          (api.minecraftservices.com/minecraft/profile)
//!   6) sessionserver.mojang.com        -> подписанные textures для ванильного клиента

use anyhow::anyhow;
use serde::Deserialize;

use crate::{error::AppError, state::AppState};

/// client_id официального Microsoft Launcher.
/// Whitelisted Mojang, не требует client_secret. См. https://wiki.vg/Microsoft_Authentication_Scheme
pub const LAUNCHER_CLIENT_ID: &str = "00000000402b5328";
pub const LAUNCHER_REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
pub const LAUNCHER_SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";

#[derive(Debug)]
pub struct MicrosoftProfile {
    pub uuid: String,                  // 32-char без дефисов
    pub username: String,
    pub skin_value: Option<String>,    // base64 textures property
    pub skin_signature: Option<String>,
}

pub async fn exchange_code(state: &AppState, code: &str) -> Result<MicrosoftProfile, AppError> {
    let legacy = state.cfg.ms_legacy_launcher;
    if !legacy && state.cfg.ms_client_id.is_empty() {
        return Err(AppError::BadRequest("Microsoft OAuth не сконфигурирован".into()));
    }

    // 1) MS access token. В legacy-режиме без client_secret, с launcher-ID.
    #[derive(Deserialize)]
    struct MsToken {
        access_token: String,
        #[serde(default)]
        id_token: Option<String>,
    }
    let mut form: Vec<(&str, &str)> = vec![
        ("code",         code),
        ("grant_type",   "authorization_code"),
    ];
    if legacy {
        form.push(("client_id",    LAUNCHER_CLIENT_ID));
        form.push(("redirect_uri", LAUNCHER_REDIRECT_URI));
        form.push(("scope",        LAUNCHER_SCOPE));
    } else {
        form.push(("client_id",     state.cfg.ms_client_id.as_str()));
        form.push(("client_secret", state.cfg.ms_client_secret.as_str()));
        form.push(("redirect_uri",  state.cfg.ms_redirect_uri.as_str()));
    }
    let ms: MsToken = state.http
        .post("https://login.live.com/oauth20_token.srf")
        .form(&form)
        .send().await.map_err(map)?
        .error_for_status().map_err(map)?
        .json().await.map_err(map)?;

    // 2) Xbox Live
    #[derive(Deserialize)]
    struct XblResp {
        #[serde(rename = "Token")] token: String,
        #[serde(rename = "DisplayClaims")] display_claims: XblClaims,
    }
    #[derive(Deserialize)] struct XblClaims { xui: Vec<XuiUhs> }
    #[derive(Deserialize)] struct XuiUhs    {
        uhs: String,
        #[serde(default)] gtg: Option<String>, // gamertag (может отсутствовать у новых аккаунтов)
    }

    // RpsTicket: legacy-режим (MBI_SSL) шлёт access_token "как есть",
    // современный (XboxLive.signin) — с префиксом "d=".
    let rps_ticket = if legacy {
        ms.access_token.clone()
    } else {
        format!("d={}", ms.access_token)
    };
    let xbl: XblResp = state.http
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": rps_ticket,
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send().await.map_err(map)?
        .error_for_status().map_err(map)?
        .json().await.map_err(map)?;
    let xui = xbl.display_claims.xui.first()
        .ok_or_else(|| AppError::Internal(anyhow!("xbl: no display claims")))?;
    let uhs = xui.uhs.clone();
    let mut gamertag = xui.gtg.clone();
    let email = ms.id_token.as_deref().and_then(extract_id_token_email);

    // Если в display claims gamertag-а не было — пробуем Xbox Profile API.
    // Тихий фолбэк: ошибки игнорируем, оставляем gamertag = None.
    if gamertag.is_none() {
        gamertag = fetch_gamertag_via_profile(state, &xbl.token, &uhs).await;
    }

    // 3) XSTS
    #[derive(Deserialize)] struct XstsResp { #[serde(rename = "Token")] token: String }
    let xsts: XstsResp = state.http
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl.token],
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send().await.map_err(map)?
        .error_for_status().map_err(map)?
        .json().await.map_err(map)?;

    // 4) Minecraft access token
    #[derive(Deserialize)] struct McAuth { access_token: String }
    let mc_resp = state.http
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", uhs, xsts.token)
        }))
        .send().await.map_err(map)?;
    if mc_resp.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(AppError::BadRequest(format!(
            "На этом Microsoft-аккаунте{who} не куплена Minecraft: Java Edition. \
             Зарегистрируйте локальный аккаунт или войдите под аккаунтом с лицензией Java.",
            who = format_who(&email, &gamertag),
        )));
    }
    let mc: McAuth = mc_resp.error_for_status().map_err(map)?.json().await.map_err(map)?;

    // 5) Профиль (uuid + ник)
    #[derive(Deserialize)] struct McProfile { id: String, name: String }
    let prof_resp = state.http
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&mc.access_token)
        .send().await.map_err(map)?;
    if prof_resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::BadRequest(format!(
            "У аккаунта{who} нет профиля Minecraft (никнейм не создан). \
             Откройте minecraft.net/profile и создайте профиль, затем войдите снова.",
            who = format_who(&email, &gamertag),
        )));
    }
    let prof: McProfile = prof_resp.error_for_status().map_err(map)?.json().await.map_err(map)?;

    // 6) Подписанные текстуры
    let (skin_value, skin_signature) = fetch_signed_textures(state, &prof.id).await;

    Ok(MicrosoftProfile {
        uuid: prof.id,
        username: prof.name,
        skin_value,
        skin_signature,
    })
}

/// Тянет gamertag через Xbox Live Profile API.
/// Требует отдельного XSTS токена для RelyingParty `http://xboxlive.com`.
/// Если у MS-аккаунта нет связанного Xbox-профиля — возвращает None.
async fn fetch_gamertag_via_profile(
    state: &AppState,
    xbl_token: &str,
    uhs: &str,
) -> Option<String> {
    #[derive(Deserialize)]
    struct Xsts { #[serde(rename = "Token")] token: String }

    let xsts: Xsts = state.http
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&serde_json::json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
            "RelyingParty": "http://xboxlive.com",
            "TokenType": "JWT"
        }))
        .send().await.ok()?
        .error_for_status().ok()?
        .json().await.ok()?;

    #[derive(Deserialize)]
    struct ProfileResp { #[serde(rename = "profileUsers")] users: Vec<ProfileUser> }
    #[derive(Deserialize)]
    struct ProfileUser { settings: Vec<Setting> }
    #[derive(Deserialize)]
    struct Setting { id: String, value: String }

    let resp: ProfileResp = state.http
        .get("https://profile.xboxlive.com/users/me/profile/settings?settings=Gamertag")
        .header("Authorization", format!("XBL3.0 x={};{}", uhs, xsts.token))
        .header("x-xbl-contract-version", "2")
        .header("Accept", "application/json")
        .send().await.ok()?
        .error_for_status().ok()?
        .json().await.ok()?;

    resp.users
        .into_iter()
        .next()?
        .settings
        .into_iter()
        .find(|s| s.id == "Gamertag")
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
}

async fn fetch_signed_textures(state: &AppState, uuid: &str) -> (Option<String>, Option<String>) {
    #[derive(Deserialize)] struct Sess { properties: Vec<Prop> }
    #[derive(Deserialize)] struct Prop { name: String, value: String, signature: Option<String> }

    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{uuid}?unsigned=false"
    );
    let resp = match state.http.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return (None, None),
    };
    let resp = match resp.error_for_status() {
        Ok(r) => r,
        Err(_) => return (None, None),
    };
    match resp.json::<Sess>().await {
        Ok(s) => {
            let tex = s.properties.into_iter().find(|p| p.name == "textures");
            match tex {
                Some(t) => (Some(t.value), t.signature),
                None => (None, None),
            }
        }
        Err(_) => (None, None),
    }
}

fn map(e: reqwest::Error) -> AppError {
    AppError::Internal(anyhow!("microsoft auth: {e}"))
}

fn format_who(email: &Option<String>, gamertag: &Option<String>) -> String {
    match (email.as_deref(), gamertag.as_deref()) {
        (Some(e), Some(g)) => format!(" ({e}, Xbox: {g})"),
        (Some(e), None)    => format!(" ({e})"),
        (None,    Some(g)) => format!(" (Xbox: {g})"),
        _ => String::new(),
    }
}

/// Декодирует payload JWT (id_token) без проверки подписи и достаёт email.
/// id_token уже подтверждён доверенным каналом (HTTPS-обменом code на token),
/// поэтому достаточно вытащить из него claims для отображения юзеру.
fn extract_id_token_email(id_token: &str) -> Option<String> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let payload_b64 = id_token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    v.get("email")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("preferred_username").and_then(|x| x.as_str()))
        .or_else(|| v.get("upn").and_then(|x| x.as_str()))
        .map(|s| s.to_string())
}
