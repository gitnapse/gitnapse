use crate::auth;
use crate::oauth_session;
use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use url::form_urlencoded::Serializer;

const ENV_OAUTH_CLIENT_ID: &str = "GITNAPSE_GITHUB_OAUTH_CLIENT_ID";
const ENV_GITHUB_CLIENT_ID: &str = "GITHUB_CLIENT_ID";
const DEFAULT_OAUTH_CLIENT_ID: &str = "Iv23liX3yGiGUEYkSlFW";

fn resolve_client_id(client_id: Option<String>) -> Result<String> {
    if let Some(cli_id) = client_id {
        let trimmed = cli_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Ok(env_id) = std::env::var(ENV_OAUTH_CLIENT_ID) {
        let trimmed = env_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Ok(env_id) = std::env::var(ENV_GITHUB_CLIENT_ID) {
        let trimmed = env_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    Ok(DEFAULT_OAUTH_CLIENT_ID.to_string())
}

fn resolve_scopes(scopes: &[String]) -> Vec<String> {
    if scopes.is_empty() {
        return vec!["read:user".to_string()];
    }
    scopes
        .iter()
        .map(|scope| scope.trim().to_string())
        .filter(|scope| !scope.is_empty())
        .collect()
}

fn terminal_hyperlink(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}

fn try_open_browser(url: &str) -> bool {
    if webbrowser::open(url).is_ok() {
        return true;
    }
    // Fallbacks for terminals/environments where webbrowser backend is unavailable.
    if cfg!(target_os = "linux") {
        if Command::new("xdg-open").arg(url).status().is_ok() {
            return true;
        }
        if Command::new("wslview").arg(url).status().is_ok() {
            return true;
        }
    } else if cfg!(target_os = "macos") {
        if Command::new("open").arg(url).status().is_ok() {
            return true;
        }
    } else if cfg!(target_os = "windows")
        && Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .is_ok()
    {
        return true;
    }
    false
}

/// Device authorization data returned by GitHub for step one of the OAuth
/// device flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFlow {
    /// Code the user types into the verification page.
    pub user_code: String,
    /// URL where the user enters [`DeviceFlow::user_code`].
    pub verification_uri: String,
    /// Opaque code used to poll for the access token.
    pub device_code: String,
    /// Minimum seconds between polls requested by GitHub.
    pub interval: u64,
    /// Lifetime of the device code, in seconds.
    pub expires_in: u64,
}

/// Result of a single [`complete_device_flow`] poll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevicePoll {
    /// The user has not authorized the device yet; keep polling.
    Pending,
    /// GitHub asked to slow down; add five seconds before the next poll.
    SlowDown,
    /// Authorization completed and the token was persisted; contains the login.
    Done(String),
    /// The user denied the authorization request.
    Denied,
    /// The device code expired before the user authorized.
    Expired,
}

struct PendingFlow {
    client_id: String,
    expires_in: u64,
    created_unix: u64,
}

impl PendingFlow {
    fn is_expired(&self) -> bool {
        self.expires_in != 0 && now_unix() >= self.created_unix.saturating_add(self.expires_in)
    }
}

enum PendingLookup {
    Known(String),
    Expired,
    Unknown,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenWire {
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn pending_flows() -> &'static Mutex<HashMap<String, PendingFlow>> {
    static PENDING_FLOWS: OnceLock<Mutex<HashMap<String, PendingFlow>>> = OnceLock::new();
    PENDING_FLOWS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn remember_device_flow(device_code: &str, client_id: String, expires_in: u64) {
    let mut guard = pending_flows()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.insert(
        device_code.to_string(),
        PendingFlow {
            client_id,
            expires_in,
            created_unix: now_unix(),
        },
    );
}

fn forget_device_flow(device_code: &str) {
    let mut guard = pending_flows()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.remove(device_code);
}

fn lookup_device_flow(device_code: &str) -> PendingLookup {
    let mut guard = pending_flows()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match guard.get(device_code) {
        Some(flow) if flow.is_expired() => {
            guard.remove(device_code);
            PendingLookup::Expired
        }
        Some(flow) => PendingLookup::Known(flow.client_id.clone()),
        None => PendingLookup::Unknown,
    }
}

/// Starts the OAuth device flow: requests a user code from GitHub and returns
/// the data needed to show it to the user.
///
/// This function prints nothing and does not require a terminal.
pub fn begin_device_flow(client_id: Option<String>, scopes: &[String]) -> Result<DeviceFlow> {
    crate::runtime::ensure_crypto_provider();
    let client_id = resolve_client_id(client_id)?;
    let scopes = resolve_scopes(scopes);
    let device_credential = SecretString::new(client_id.clone().into());
    let runtime = crate::runtime::get_runtime();

    let device_codes = runtime
        .block_on(async {
            let crab = octocrab::Octocrab::builder()
                .base_uri("https://github.com")
                .context("Cannot set OAuth base URI")?
                .add_header(ACCEPT, "application/json".to_string())
                .build()
                .context("Cannot create OAuth client")?;

            crab.authenticate_as_device(&device_credential, scopes.iter().map(String::as_str))
                .await
                .context("Unable to request OAuth device codes from GitHub")
        })
        .context("Unable to request OAuth device codes from GitHub")?;

    let flow = DeviceFlow {
        user_code: device_codes.user_code,
        verification_uri: device_codes.verification_uri,
        device_code: device_codes.device_code,
        interval: device_codes.interval,
        expires_in: device_codes.expires_in,
    };

    remember_device_flow(&flow.device_code, client_id, flow.expires_in);
    Ok(flow)
}

/// Performs a single poll of the OAuth device flow for `device_code`.
///
/// On success (`DevicePoll::Done`) the access token and session metadata are
/// persisted through [`auth::save_token`] and [`oauth_session::save_session`],
/// and the authenticated login is returned. This function prints nothing and
/// does not require a terminal.
pub fn complete_device_flow(device_code: &str) -> Result<DevicePoll> {
    let device_code = device_code.trim();
    if device_code.is_empty() {
        return Err(anyhow!("device code is empty"));
    }
    crate::runtime::ensure_crypto_provider();

    let client_id = match lookup_device_flow(device_code) {
        PendingLookup::Known(client_id) => client_id,
        PendingLookup::Expired => return Ok(DevicePoll::Expired),
        PendingLookup::Unknown => resolve_client_id(None)?,
    };

    let wire = request_device_token(&client_id, device_code)?;

    if let Some(error) = wire.error.as_deref() {
        return match error {
            "authorization_pending" => Ok(DevicePoll::Pending),
            "slow_down" => Ok(DevicePoll::SlowDown),
            "access_denied" => {
                forget_device_flow(device_code);
                Ok(DevicePoll::Denied)
            }
            "expired_token" => {
                forget_device_flow(device_code);
                Ok(DevicePoll::Expired)
            }
            other => Err(anyhow!(
                "OAuth device flow failed: {}",
                wire.error_description.as_deref().unwrap_or(other)
            )),
        };
    }

    let Some(access_token) = wire
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
    else {
        return Err(anyhow!(
            "OAuth device token response did not include an access token"
        ));
    };

    persist_device_tokens(&wire, &client_id, access_token)?;
    forget_device_flow(device_code);
    Ok(DevicePoll::Done(fetch_login(access_token)))
}

fn request_device_token(client_id: &str, device_code: &str) -> Result<DeviceTokenWire> {
    crate::runtime::get_runtime().block_on(async {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gitnapse/0.1"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Cannot build OAuth device HTTP client")?;

        let body = Serializer::new(String::new())
            .append_pair("client_id", client_id)
            .append_pair("device_code", device_code)
            .append_pair("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
            .finish();

        let response = client
            .post("https://github.com/login/oauth/access_token")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .context("OAuth device token request failed")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "OAuth device token request failed with status {}",
                response.status()
            ));
        }

        response
            .json::<DeviceTokenWire>()
            .await
            .context("Invalid OAuth device token response")
    })
}

fn persist_device_tokens(
    wire: &DeviceTokenWire,
    client_id: &str,
    access_token: &str,
) -> Result<()> {
    let now = now_unix();
    let session = oauth_session::OAuthSession {
        access_token: access_token.to_string(),
        token_type: wire
            .token_type
            .clone()
            .unwrap_or_else(|| "bearer".to_string()),
        scope: wire
            .scope
            .clone()
            .unwrap_or_default()
            .split(',')
            .map(|scope| scope.trim().to_string())
            .filter(|scope| !scope.is_empty())
            .collect(),
        expires_at_unix: wire.expires_in.map(|seconds| now.saturating_add(seconds)),
        refresh_token: wire.refresh_token.clone(),
        refresh_expires_at_unix: wire
            .refresh_token_expires_in
            .map(|seconds| now.saturating_add(seconds)),
        client_id: client_id.to_string(),
    };

    auth::save_token(access_token).context("Cannot store OAuth access token")?;
    oauth_session::save_session(&session).context("Cannot store OAuth session metadata")?;
    Ok(())
}

fn fetch_login(access_token: &str) -> String {
    crate::provider::create_provider(crate::provider::ProviderKind::GitHub, Some(access_token))
        .ok()
        .and_then(|provider| provider.fetch_authenticated_user().ok().flatten())
        .unwrap_or_else(|| "unknown user".to_string())
}

pub fn oauth_device_login_cli(
    client_id: Option<String>,
    scopes: Vec<String>,
    timeout_secs: u64,
) -> Result<()> {
    let scopes = resolve_scopes(&scopes);
    let flow = begin_device_flow(client_id, &scopes)?;

    println!("OAuth device login started.");
    let opened = try_open_browser(&flow.verification_uri);
    if opened {
        println!("1. Browser launch requested automatically.");
        println!("   If no browser appears, open this URL manually.");
    }
    println!(
        "1. Open this URL in your browser: {}",
        flow.verification_uri
    );
    println!(
        "   Clickable link (if your terminal supports OSC8): {}",
        terminal_hyperlink(&flow.verification_uri)
    );
    println!("2. Enter code: {}", flow.user_code);
    println!("3. After authorization, keep this terminal open while token exchange completes.");
    println!("Scopes requested: {}", scopes.join(","));

    let timeout = Duration::from_secs(timeout_secs.max(60));
    let deadline = Instant::now() + timeout;
    let mut interval = Duration::from_secs(flow.interval.max(1));

    loop {
        match complete_device_flow(&flow.device_code)? {
            DevicePoll::Done(login) => {
                println!("OAuth login completed. Token saved securely for user: {login}");
                return Ok(());
            }
            DevicePoll::Pending => {}
            DevicePoll::SlowDown => interval += Duration::from_secs(5),
            DevicePoll::Denied => {
                return Err(anyhow!("OAuth device flow was denied by the user."));
            }
            DevicePoll::Expired => {
                return Err(anyhow!("OAuth device code expired before authorization."));
            }
        }

        if Instant::now() >= deadline {
            return Err(anyhow!(
                "OAuth device flow timed out after {} seconds.",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(interval.min(deadline.saturating_duration_since(Instant::now())));
    }
}

pub fn oauth_status_cli() -> Result<()> {
    let token = auth::load_token()?;
    let oauth_session_present = oauth_session::load_session()?.is_some();

    if token.is_none() {
        println!("oauth_logged_in=false");
        println!("authenticated=false");
        println!("oauth_session_present={oauth_session_present}");
        return Ok(());
    }

    let client =
        crate::provider::create_provider(crate::provider::ProviderKind::GitHub, token.as_deref())?;
    let user = client.fetch_authenticated_user()?;
    let authenticated = user.is_some();

    println!("oauth_logged_in={}", oauth_session_present && authenticated);
    println!("authenticated={authenticated}");
    println!("oauth_session_present={oauth_session_present}");
    if let Some(login) = user {
        println!("user={login}");
    }
    Ok(())
}
