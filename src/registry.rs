//! Client for the GitNapse themes registry (`github.com/gitnapse/themes`).
//!
//! The registry is a data repository: `index.json` at its root is the
//! machine-readable API that lists available themes and where each one lives
//! (under `colors/`). No authentication is needed — the repo is public and we
//! fetch from raw.githubusercontent.com.

use anyhow::{Context, Result, anyhow};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

pub const REGISTRY_OWNER: &str = "gitnapse";
pub const REGISTRY_REPO: &str = "themes";
pub const REGISTRY_BRANCH: &str = "main";

fn raw_base() -> String {
    format!("https://raw.githubusercontent.com/{REGISTRY_OWNER}/{REGISTRY_REPO}/{REGISTRY_BRANCH}")
}

/// URL of the registry index (the themes "API").
pub fn index_url() -> String {
    format!("{}/index.json", raw_base())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeIndex {
    pub version: u32,
    #[serde(default)]
    pub base_url: String,
    pub themes: Vec<ThemeEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeEntry {
    pub name: String,
    /// Path of the theme file inside the repo (or a full URL override).
    pub file: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub dark: bool,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
}

impl ThemeEntry {
    /// Resolve the raw download URL for this theme's file.
    pub fn download_url(&self) -> String {
        if self.file.starts_with("http://") || self.file.starts_with("https://") {
            self.file.clone()
        } else {
            format!("{}/{}", raw_base(), self.file)
        }
    }
}

fn http_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("gitnapse"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

/// Fetch the registry index (list of available themes).
pub fn fetch_index() -> Result<ThemeIndex> {
    let client = http_client()?;
    let url = index_url();
    let body = crate::runtime::get_runtime().block_on(async {
        client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    })?;
    serde_json::from_str(&body).with_context(|| format!("invalid registry index at {url}"))
}

/// List themes available in the registry.
pub fn list_remote() -> Result<Vec<ThemeEntry>> {
    Ok(fetch_index()?.themes)
}

/// Find a theme by name (case-insensitive).
pub fn find_theme<'a>(index: &'a ThemeIndex, name: &str) -> Option<&'a ThemeEntry> {
    index
        .themes
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

/// Download the raw JSONC content of a theme file.
pub fn download_theme(entry: &ThemeEntry) -> Result<String> {
    let client = http_client()?;
    let url = entry.download_url();
    crate::runtime::get_runtime()
        .block_on(async {
            client
                .get(&url)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await
        })
        .with_context(|| format!("cannot download theme from {url}"))
}

/// Install a named theme from the registry into the user config themes dir.
/// Returns the installed entry.
pub fn install_theme(name: &str) -> Result<ThemeEntry> {
    let index = fetch_index()?;
    let entry = find_theme(&index, name)
        .cloned()
        .ok_or_else(|| anyhow!("theme '{name}' not found in the registry"))?;
    let content = download_theme(&entry)?;
    let dest = crate::config::config_dir()?
        .join("themes")
        .join(format!("{}.jsonc", entry.name));
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dest, content)
        .with_context(|| format!("cannot write theme to {}", dest.display()))?;
    Ok(entry)
}

/// Themes present in the user config themes directory.
pub fn list_installed() -> Result<Vec<String>> {
    let dir = crate::config::config_dir()?.join("themes");
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "jsonc")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Remove an installed theme file. Returns whether it existed.
pub fn uninstall_theme(name: &str) -> Result<bool> {
    let path = crate::config::config_dir()?
        .join("themes")
        .join(format!("{name}.jsonc"));
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("cannot remove {}", path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, file: &str) -> ThemeEntry {
        ThemeEntry {
            name: name.into(),
            file: file.into(),
            description: None,
            dark: false,
            background: None,
            foreground: None,
        }
    }

    #[test]
    fn download_url_prefers_explicit_http() {
        let e = entry("x", "https://cdn.example.com/t.jsonc");
        assert_eq!(e.download_url(), "https://cdn.example.com/t.jsonc");
    }

    #[test]
    fn download_url_builds_raw_path() {
        let e = entry("X", "colors/X.jsonc");
        assert_eq!(
            e.download_url(),
            "https://raw.githubusercontent.com/gitnapse/themes/main/colors/X.jsonc"
        );
    }

    #[test]
    fn find_theme_is_case_insensitive() {
        let index = ThemeIndex {
            version: 1,
            base_url: String::new(),
            themes: vec![entry("Madrid", "colors/Madrid.jsonc")],
        };
        assert!(find_theme(&index, "madrid").is_some());
        assert!(find_theme(&index, "nope").is_none());
    }
}
