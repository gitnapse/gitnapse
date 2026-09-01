pub mod account;
pub mod keybindings;
pub mod theme;

pub use account::AccountConfig;
pub use keybindings::KeybindingsConfig;
pub use theme::ThemeColors;
pub use theme::ThemeConfig;

use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Returns the path to the account configuration file, creating the config
/// directory if it does not exist.
///
/// # Errors
/// Returns an error if the project config directory cannot be resolved or created.
pub fn config_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
        .ok_or_else(|| anyhow!("Unable to resolve project config directory"))?;
    let dir = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&dir)
        .with_context(|| format!("Cannot create config directory: {}", dir.display()))?;
    Ok(dir)
}

pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("account.json"))
}

pub(crate) fn strip_jsonc_comments(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            // Remove inline // comments (but not those inside strings)
            let mut in_string = false;
            let mut prev = ' ';
            let mut result = String::with_capacity(line.len());
            for ch in line.chars() {
                if ch == '"' && prev != '\\' {
                    in_string = !in_string;
                }
                if !in_string && ch == '/' && prev == '/' {
                    // Remove the // and everything after, plus trailing space
                    result.pop();
                    break;
                }
                if !in_string || ch != '/' || prev != '/' {
                    prev = ch;
                    result.push(ch);
                }
            }
            result.trim_end().to_string()
        })
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}
