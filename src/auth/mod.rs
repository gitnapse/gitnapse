pub mod oauth;
pub mod oauth_session;
pub mod secure_store;

use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const ENV_TOKEN: &str = "GITHUB_TOKEN";
const ENV_OAUTH_CLIENT_ID: &str = "GITNAPSE_GITHUB_OAUTH_CLIENT_ID";
const ENV_GITHUB_CLIENT_ID: &str = "GITHUB_CLIENT_ID";
const DEFAULT_OAUTH_CLIENT_ID: &str = "Iv23liX3yGiGUEYkSlFW";
const TOKEN_SECRET_KEY: &str = "github_token";

fn token_file() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
        .ok_or_else(|| anyhow!("Unable to resolve project config directory"))?;
    let dir = project_dirs.config_dir();
    fs::create_dir_all(dir)
        .with_context(|| format!("Cannot create config dir: {}", dir.display()))?;
    Ok(dir.join("token"))
}

pub fn load_token() -> Result<Option<String>> {
    if let Ok(env_token) = std::env::var(ENV_TOKEN) {
        let trimmed = env_token.trim().to_owned();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed));
        }
    }

    if let Some(session_token) = oauth_session::resolve_access_token()? {
        let trimmed = session_token.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed));
        }
    }

    let file = token_file()?;
    secure_store::load_secret(TOKEN_SECRET_KEY, &file).map_err(|e| anyhow!("{e}"))
}

pub fn save_token(token: &str) -> Result<()> {
    let token = token.trim();
    if token.is_empty() {
        return Err(anyhow!("Token is empty"));
    }

    let file = token_file()?;
    let _ =
        secure_store::save_secret(TOKEN_SECRET_KEY, &file, token).map_err(|e| anyhow!("{e}"))?;

    Ok(())
}

pub fn clear_token() -> Result<()> {
    let file = token_file()?;
    secure_store::clear_secret(TOKEN_SECRET_KEY, &file).map_err(|e| anyhow!("{e}"))?;
    let _ = oauth_session::clear_session();
    Ok(())
}

pub fn set_token_cli(token_arg: Option<String>) -> Result<()> {
    let token = match token_arg {
        Some(t) => t,
        None => {
            print!("GitHub token: ");
            io::stdout().flush().context("Cannot flush stdout")?;
            rpassword::read_password().context("Cannot read token from terminal")?
        }
    };

    save_token(&token)?;
    println!("Token saved successfully.");
    Ok(())
}

pub fn clear_token_cli() -> Result<()> {
    clear_token()?;
    println!("Stored token removed.");
    Ok(())
}

pub fn status_cli() -> Result<()> {
    let env_ok = std::env::var(ENV_TOKEN).is_ok_and(|t| !t.trim().is_empty());
    let oauth_client_id_ok = std::env::var(ENV_OAUTH_CLIENT_ID).is_ok_and(|t| !t.trim().is_empty());
    let github_client_id_ok =
        std::env::var(ENV_GITHUB_CLIENT_ID).is_ok_and(|t| !t.trim().is_empty());
    let file = token_file()?;
    let file_ok = file.exists();
    let oauth_session_ok = oauth_session::load_session()?.is_some();

    println!("Authentication status:");
    println!(
        "- ENV {ENV_TOKEN}: {}",
        if env_ok { "available" } else { "missing" }
    );
    println!(
        "- Stored token file: {} ({})",
        file.display(),
        if file_ok { "present" } else { "missing" }
    );
    println!(
        "- ENV {ENV_OAUTH_CLIENT_ID}: {}",
        if oauth_client_id_ok {
            "available"
        } else {
            "missing"
        }
    );
    println!(
        "- ENV {ENV_GITHUB_CLIENT_ID}: {}",
        if github_client_id_ok {
            "available"
        } else {
            "missing"
        }
    );
    println!("- Built-in OAuth Client ID: {}", DEFAULT_OAUTH_CLIENT_ID);
    println!(
        "- OAuth session file: {}",
        if oauth_session_ok {
            "present"
        } else {
            "missing"
        }
    );
    println!(
        "- Secret storage mode (preferred): {}",
        secure_store::preferred_backend_name()
    );
    Ok(())
}
