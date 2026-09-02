//! Client for the GitNapse themes registry (`github.com/gitnapse/themes`).
//!
//! The registry is a data repository: `index.json` at its root is the
//! machine-readable API that lists available themes and where each one lives
//! (under `colors/`). No authentication is needed — the repo is public and we
//! fetch from raw.githubusercontent.com.
//!
//! The index is treated as untrusted input: theme names become file names on
//! disk and `file` values become URLs, so both are validated before use.

use anyhow::{Context, Result, anyhow};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

pub const REGISTRY_OWNER: &str = "gitnapse";
pub const REGISTRY_REPO: &str = "themes";
pub const REGISTRY_BRANCH: &str = "main";

const MAX_NAME_LEN: usize = 64;
const MAX_FILE_LEN: usize = 256;

fn raw_base() -> String {
    format!("https://raw.githubusercontent.com/{REGISTRY_OWNER}/{REGISTRY_REPO}/{REGISTRY_BRANCH}")
}

/// URL of the registry index (the themes "API").
pub fn index_url() -> String {
    format!("{}/index.json", raw_base())
}

/// Validate a theme name before it becomes part of a local file name.
/// Allows ASCII alphanumerics plus `-`, `_`, `.` and spaces; rejects anything
/// that could traverse directories (`/`, `\`, `:`, `..` tricks, NUL).
pub fn valid_theme_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_LEN
        && !name.contains(['/', '\\', ':'])
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
}

/// Validate the `file` value of a theme entry: either an `https://` URL or a
/// relative path inside the registry repo. Rejects plain `http://` (SSRF risk
/// against localhost), absolute paths and `..` traversal components.
pub fn valid_theme_file(file: &str) -> bool {
    if file.is_empty() || file.len() > MAX_FILE_LEN || file.contains('\\') {
        return false;
    }
    if let Some(rest) = file.strip_prefix("https://") {
        return !rest.is_empty() && !rest.starts_with('/') && !rest.contains([' ', '\n', '\r']);
    }
    if file.contains("://") || file.starts_with('/') {
        return false;
    }
    file.split('/').all(|seg| seg != ".." && !seg.is_empty())
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
        if self.file.starts_with("https://") {
            self.file.clone()
        } else {
            format!("{}/{}", raw_base(), self.file)
        }
    }
}

/// Validate the entries of a fetched index. Returns the list of invalid
/// entries so the caller can reject a compromised registry early.
fn invalid_entries(index: &ThemeIndex) -> Vec<&ThemeEntry> {
    index
        .themes
        .iter()
        .filter(|t| !valid_theme_name(&t.name) || !valid_theme_file(&t.file))
        .collect()
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
    let index: ThemeIndex =
        serde_json::from_str(&body).with_context(|| format!("invalid registry index at {url}"))?;
    let bad = invalid_entries(&index);
    if !bad.is_empty() {
        let names: Vec<&str> = bad.iter().map(|t| t.name.as_str()).collect();
        return Err(anyhow!(
            "registry index at {url} contains invalid theme entries: {}",
            names.join(", ")
        ));
    }
    Ok(index)
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
    if !valid_theme_name(name) {
        return Err(anyhow!("invalid theme name '{name}'"));
    }
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
    fn download_url_prefers_explicit_https() {
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

    #[test]
    fn theme_names_must_be_safe_filenames() {
        assert!(valid_theme_name("Madrid"));
        assert!(valid_theme_name("X"));
        assert!(valid_theme_name("My Theme 2"));
        assert!(!valid_theme_name(""));
        assert!(!valid_theme_name(".."));
        assert!(!valid_theme_name("."));
        assert!(!valid_theme_name("../evil"));
        assert!(!valid_theme_name("a/b"));
        assert!(!valid_theme_name("a\\b"));
        assert!(!valid_theme_name("a:b"));
        assert!(!valid_theme_name(&"a".repeat(65)));
    }

    #[test]
    fn theme_files_must_be_safe_paths_or_https() {
        assert!(valid_theme_file("colors/X.jsonc"));
        assert!(valid_theme_file("https://cdn.example.com/t.jsonc"));
        assert!(!valid_theme_file(""));
        assert!(!valid_theme_file("../evil.jsonc"));
        assert!(!valid_theme_file("colors/../../evil.jsonc"));
        assert!(!valid_theme_file("/etc/passwd"));
        assert!(!valid_theme_file("a\\b"));
        assert!(!valid_theme_file("http://localhost:8787/t.jsonc"));
        assert!(!valid_theme_file("https://"));
        assert!(!valid_theme_file(&"a/".repeat(130)));
    }

    #[test]
    fn invalid_entries_are_reported() {
        let index = ThemeIndex {
            version: 1,
            base_url: String::new(),
            themes: vec![
                entry("ok", "colors/ok.jsonc"),
                entry("../evil", "colors/x.jsonc"),
                entry("ok2", "http://x/y.jsonc"),
            ],
        };
        let bad: Vec<&str> = invalid_entries(&index)
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(bad, ["../evil", "ok2"]);
    }
}
