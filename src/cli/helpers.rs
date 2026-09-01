use anyhow::{Context, Result, anyhow};
use std::process::Command;

use crate::auth;
use crate::error::GitHubError;
use crate::provider::GitProvider;
use std::sync::Arc;

// ── Git helpers ─────────────────────────────────────────────────────────

pub fn is_git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

pub fn check_git() -> Result<()> {
    if !is_git_available() {
        return Err(anyhow!(
            "git is not installed or not in PATH\n\
             Install git: https://git-scm.com/downloads"
        ));
    }
    Ok(())
}

pub fn run_git(args: &[&str]) -> Result<std::process::Output> {
    check_git()?;
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to execute: git {}", args.join(" ")))
}

pub fn run_git_with_cwd(args: &[&str], cwd: &std::path::Path) -> Result<std::process::Output> {
    check_git()?;
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to execute: git {} in {:?}", args.join(" "), cwd))
}

pub fn stderr_msg(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

pub fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn not_a_repo_msg() -> String {
    "not a git repository — run this from inside a git repository".to_string()
}

pub fn not_a_repo_or_stderr(output: &std::process::Output, fallback_prefix: &str) -> String {
    let msg = stderr_msg(output);
    if msg.contains("not a git repository") {
        not_a_repo_msg()
    } else {
        format!("{fallback_prefix}:\n{msg}")
    }
}

// ── Repo detection ──────────────────────────────────────────────────────

pub fn detect_repo_from_remote() -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let url = url.strip_suffix(".git").unwrap_or(&url);

    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git@"))
        .unwrap_or(url);

    let pos = after_scheme.find(':').or_else(|| after_scheme.find('/'))?;
    let path = &after_scheme[pos + 1..];

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else {
        None
    }
}

pub fn resolve_full_name(repo: &str) -> Result<String> {
    if repo.contains('/') {
        Ok(repo.to_string())
    } else if let Some(detected) = detect_repo_from_remote() {
        Ok(detected)
    } else {
        Err(anyhow!(
            "repository name '{repo}' is ambiguous. Use <owner/repo> or run from inside a cloned repo."
        ))
    }
}

// ── API helpers ─────────────────────────────────────────────────────────

pub fn handle_api_error(full_name: &str, e: &anyhow::Error) -> String {
    e.downcast_ref::<GitHubError>()
        .map(|gh_err| match gh_err {
            GitHubError::Api { status, body } if *status == 404 || body.contains("Not Found") => {
                format!("repository '{full_name}' not found on GitHub")
            }
            GitHubError::Unauthorized => {
                "authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'"
                    .to_string()
            }
            GitHubError::RateLimited {
                remaining: _,
                reset,
            } => {
                format!("GitHub API rate limit exceeded — resets at timestamp {reset}")
            }
            _ => format!("{gh_err}"),
        })
        .unwrap_or_else(|| format!("{e}"))
}

pub fn make_client() -> Result<Arc<dyn GitProvider>> {
    let token = auth::load_token()?;
    crate::provider::create_provider(crate::provider::ProviderKind::GitHub, token.as_deref())
        .map_err(|e| anyhow!("failed to create HTTP client: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_available() {
        // git should be available in the test environment
        assert!(is_git_available());
    }

    #[test]
    fn test_check_git_succeeds() {
        assert!(check_git().is_ok());
    }

    #[test]
    fn test_run_git_version() {
        let out = run_git(&["--version"]);
        assert!(out.is_ok());
        let out = out.unwrap();
        assert!(out.status.success());
        let stdout = stdout_str(&out);
        assert!(stdout.contains("git version"));
    }

    #[test]
    fn test_not_a_repo_msg_format() {
        let msg = not_a_repo_msg();
        assert!(msg.contains("not a git repository"));
    }

    #[test]
    fn test_not_a_repo_or_stderr_detects_repo_msg() {
        let out = std::process::Output {
            stdout: Vec::new(),
            stderr: b"fatal: not a git repository (or any parent directory)"[..].to_vec(),
            status: std::process::Command::new("sh")
                .arg("-c")
                .arg("exit 128")
                .status()
                .unwrap(),
        };
        let msg = not_a_repo_or_stderr(&out, "git status failed");
        assert_eq!(msg, not_a_repo_msg());
    }

    #[test]
    fn test_stderr_msg_extracts_stderr() {
        let out = std::process::Output {
            stdout: Vec::new(),
            stderr: b"error message"[..].to_vec(),
            status: std::process::Command::new("true").status().unwrap(),
        };
        assert_eq!(stderr_msg(&out), "error message");
    }

    #[test]
    fn test_stdout_str_extracts_stdout() {
        let out = std::process::Output {
            stdout: b"hello\nworld\n"[..].to_vec(),
            stderr: Vec::new(),
            status: std::process::Command::new("true").status().unwrap(),
        };
        assert_eq!(stdout_str(&out), "hello\nworld\n");
    }

    #[test]
    #[serial_test::serial]
    fn test_detect_repo_from_remote_in_temp_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("test-repo");
        std::fs::create_dir(&repo_path).unwrap();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/owner/my-repo.git",
            ])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let prev = std::env::current_dir().ok();
        std::env::set_current_dir(&repo_path).unwrap();
        let result = detect_repo_from_remote();
        if let Some(d) = prev {
            let _ = std::env::set_current_dir(d);
        }

        assert_eq!(result, Some("owner/my-repo".to_string()));
    }

    #[test]
    fn test_resolve_full_name_with_slash() {
        assert_eq!(resolve_full_name("owner/repo").unwrap(), "owner/repo");
    }

    #[test]
    fn test_resolve_full_name_ambiguous_outside_repo() {
        // Only test if we're not inside a git repo with an origin remote
        if detect_repo_from_remote().is_some() {
            eprintln!(
                "skipping test_resolve_full_name_ambiguous_outside_repo: inside a git repo with origin"
            );
            return;
        }
        let err = resolve_full_name("justname").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("ambiguous"));
    }

    #[test]
    fn test_handle_api_error_404() {
        let err = anyhow::Error::from(GitHubError::Api {
            status: 404,
            body: "Not Found".to_string(),
        });
        let msg = handle_api_error("test/repo", &err);
        assert_eq!(msg, "repository 'test/repo' not found on GitHub");
    }

    #[test]
    fn test_handle_api_error_unauthorized() {
        let err: anyhow::Error = anyhow::Error::from(GitHubError::Unauthorized);
        let msg = handle_api_error("test/repo", &err);
        assert!(msg.contains("authentication required"));
    }

    #[test]
    fn test_handle_api_error_rate_limited() {
        let err = anyhow::Error::from(GitHubError::RateLimited {
            remaining: 0,
            reset: 12345,
        });
        let msg = handle_api_error("test/repo", &err);
        assert!(msg.contains("rate limit exceeded"));
    }
}
