use crate::auth::secure_store;
use crate::runtime;
use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use reqwest::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use secrecy::{ExposeSecret, zeroize::Zeroize};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded::Serializer;

const SESSION_FILE: &str = "oauth_session.json";
const SESSION_SECRET_KEY: &str = "oauth_session_json";
const ENV_OAUTH_CLIENT_SECRET: &str = "GITNAPSE_GITHUB_OAUTH_CLIENT_SECRET";
const ENV_GITHUB_CLIENT_SECRET: &str = "GITHUB_CLIENT_SECRET";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSession {
    pub access_token: String,
    pub token_type: String,
    pub scope: Vec<String>,
    pub expires_at_unix: Option<u64>,
    pub refresh_token: Option<String>,
    pub refresh_expires_at_unix: Option<u64>,
    pub client_id: String,
}

impl Drop for OAuthSession {
    fn drop(&mut self) {
        self.access_token.zeroize();
        if let Some(ref mut rt) = self.refresh_token {
            rt.zeroize();
        }
    }
}

#[derive(Debug, Deserialize)]
struct RefreshWire {
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
    error: Option<String>,
    _error_description: Option<String>,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn session_file() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
        .ok_or_else(|| anyhow!("Unable to resolve project config directory"))?;
    fs::create_dir_all(dirs.config_dir()).with_context(|| {
        format!(
            "Cannot create config directory: {}",
            dirs.config_dir().display()
        )
    })?;
    Ok(Path::new(dirs.config_dir()).join(SESSION_FILE))
}

pub fn save_from_oauth(oauth: &octocrab::auth::OAuth, client_id: &str) -> Result<()> {
    let now = now_unix();
    let session = OAuthSession {
        access_token: oauth.access_token.expose_secret().to_string(),
        token_type: oauth.token_type.clone(),
        scope: oauth.scope.clone(),
        expires_at_unix: oauth.expires_in.map(|s| now.saturating_add(s as u64)),
        refresh_token: oauth
            .refresh_token
            .as_ref()
            .map(|value| value.expose_secret().to_string()),
        refresh_expires_at_unix: oauth
            .refresh_token_expires_in
            .map(|s| now.saturating_add(s as u64)),
        client_id: client_id.to_string(),
    };
    save_session(&session)
}

pub fn save_session(session: &OAuthSession) -> Result<()> {
    let file = session_file()?;
    let content =
        serde_json::to_string_pretty(session).context("Cannot serialize OAuth session")?;
    let _ = secure_store::save_secret(SESSION_SECRET_KEY, &file, &content)?;
    Ok(())
}

pub fn clear_session() -> Result<()> {
    let file = session_file()?;
    secure_store::clear_secret(SESSION_SECRET_KEY, &file)?;
    Ok(())
}

pub fn load_session() -> Result<Option<OAuthSession>> {
    let file = session_file()?;
    let Some(raw) = secure_store::load_secret(SESSION_SECRET_KEY, &file)? else {
        return Ok(None);
    };
    let session: OAuthSession =
        serde_json::from_str(&raw).context("Invalid OAuth session format")?;
    Ok(Some(session))
}

pub fn resolve_access_token() -> Result<Option<String>> {
    let Some(mut session) = load_session()? else {
        return Ok(None);
    };

    // If still valid (or no expiry metadata), use it directly.
    let now = now_unix();
    let about_to_expire = session
        .expires_at_unix
        .map(|exp| exp <= now.saturating_add(60))
        .unwrap_or(false);
    if !about_to_expire {
        return Ok(Some(session.access_token.clone()));
    }

    // If expiring/expired, attempt refresh when possible.
    if let Some(refreshed) = try_refresh(&session)? {
        session = refreshed;
        save_session(&session)?;
        return Ok(Some(session.access_token.clone()));
    }

    // No refresh available; caller can fallback to legacy token file.
    Ok(Some(session.access_token.clone()))
}

fn try_refresh(session: &OAuthSession) -> Result<Option<OAuthSession>> {
    let Some(refresh_token) = session
        .refresh_token
        .as_ref()
        .filter(|t| !t.trim().is_empty())
    else {
        return Ok(None);
    };

    let now = now_unix();
    if session
        .refresh_expires_at_unix
        .map(|exp| exp <= now.saturating_add(60))
        .unwrap_or(false)
    {
        return Ok(None);
    }

    let client_secret = std::env::var(ENV_OAUTH_CLIENT_SECRET)
        .ok()
        .or_else(|| std::env::var(ENV_GITHUB_CLIENT_SECRET).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let Some(client_secret) = client_secret else {
        return Ok(None);
    };

    let client_id = session.client_id.clone();
    let refresh_token = refresh_token.clone();

    runtime::get_runtime().block_on(async {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gitnapse/0.1"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Cannot build OAuth refresh HTTP client")?;

        let body = Serializer::new(String::new())
            .append_pair("client_id", &client_id)
            .append_pair("client_secret", &client_secret)
            .append_pair("grant_type", "refresh_token")
            .append_pair("refresh_token", &refresh_token)
            .finish();

        let response = client
            .post("https://github.com/login/oauth/access_token")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .context("OAuth refresh request failed")?;

        if !response.status().is_success() {
            return Ok(None);
        }
        let wire: RefreshWire = response
            .json()
            .await
            .context("Invalid OAuth refresh response")?;
        if wire.error.is_some() {
            return Ok(None);
        }
        let Some(access_token) = wire.access_token.clone().filter(|s| !s.trim().is_empty()) else {
            return Ok(None);
        };

        let scope = wire
            .scope
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        let now = now_unix();
        Ok(Some(OAuthSession {
            access_token,
            token_type: wire.token_type.unwrap_or_else(|| "bearer".to_string()),
            scope,
            expires_at_unix: wire.expires_in.map(|s| now.saturating_add(s)),
            refresh_token: wire.refresh_token.or(Some(refresh_token)),
            refresh_expires_at_unix: wire.refresh_token_expires_in.map(|s| now.saturating_add(s)),
            client_id,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::OAuthSession;

    #[test]
    fn session_serialization_roundtrip() {
        let session = OAuthSession {
            access_token: "a".to_string(),
            token_type: "bearer".to_string(),
            scope: vec!["read:user".to_string()],
            expires_at_unix: Some(123),
            refresh_token: Some("r".to_string()),
            refresh_expires_at_unix: Some(456),
            client_id: "cid".to_string(),
        };
        let text = serde_json::to_string(&session).expect("serialize");
        let parsed: OAuthSession = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(parsed.client_id, "cid");
        assert_eq!(parsed.scope.len(), 1);
    }
}
