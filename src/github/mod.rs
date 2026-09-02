use crate::error::GitHubError;
use crate::runtime;
use reqwest::Client;
use reqwest::Response;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

mod ci;
mod compare;
mod content;
mod provider_impl;
mod prs;
mod releases;
mod repos;

pub(crate) const GITHUB_API: &str = "https://api.github.com";

// ── Retry helpers ────────────────────────────────────────────────────

/// Retry a fallible operation up to 3 times when it fails with a network error
/// (for functions that use [`GitHubError`]).
///
/// Non‑network errors are propagated immediately. A short sleep is inserted
/// between retries.
pub(crate) async fn with_retry<F, Fut, T>(f: F) -> Result<T, GitHubError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, GitHubError>>,
{
    let mut last_err = None;
    for attempt in 0..3 {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if matches!(&e, GitHubError::Network(_)) && attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(last_err.unwrap_or(GitHubError::Other("Retry exhausted".into())))
}

// ── Client ───────────────────────────────────────────────────────────

pub struct GitHubClient {
    pub(crate) client: Client,
    rate_limit_remaining: Mutex<Option<u32>>,
    rate_limit_reset: Mutex<Option<u64>>,
}

/// Parsed representation of an `@me` / `me:` query.
#[derive(Debug, Clone)]
pub(crate) struct MeQuery {
    pub(crate) text_terms: Vec<String>,
    pub(crate) languages: Vec<String>,
}

impl GitHubClient {
    pub(crate) fn api_base() -> String {
        std::env::var("GITNAPSE_GITHUB_API")
            .ok()
            .map(|v| v.trim().trim_end_matches('/').to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| GITHUB_API.to_string())
    }

    /// Parse a `@me` or `me:` query into structured terms and languages.
    ///
    /// Recognised forms:
    ///   - `@me`                         — all authenticated repos
    ///   - `@me   rust`                  — repos matching "rust" (any whitespace after @me)
    ///   - `@me language:rust,go`        — filter by language(s)
    ///   - `me:rust`                     — shorthand me: prefix
    ///
    /// Returns `None` when the query does **not** start with `@me` / `me:`.
    /// `@me,rust` or `@mex` are *not* treated as `@me` queries.
    pub(crate) fn parse_me_query(query: &str) -> Option<MeQuery> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return None;
        }

        let rest = if trimmed.eq_ignore_ascii_case("@me") {
            ""
        } else if trimmed.len() >= 3
            && trimmed[..3].eq_ignore_ascii_case("@me")
            && (trimmed.len() == 3 || trimmed[3..].starts_with(|c: char| c.is_whitespace()))
        {
            // @me followed by whitespace (or exact @me caught above)
            trimmed[3..].trim()
        } else {
            // me: prefix — rest may be empty (e.g. just "me:")
            trimmed.strip_prefix("me:")?.trim()
        };

        let mut text_terms = Vec::new();
        let mut languages = Vec::new();
        for raw in rest.split_whitespace() {
            if let Some(lang_expr) = raw
                .strip_prefix("language:")
                .or_else(|| raw.strip_prefix("lang:"))
            {
                for lang in lang_expr.split(',') {
                    let lang = lang.trim().to_lowercase();
                    if !lang.is_empty() {
                        languages.push(lang);
                    }
                }
            } else {
                let term = raw.trim().to_lowercase();
                if !term.is_empty() {
                    text_terms.push(term);
                }
            }
        }

        Some(MeQuery {
            text_terms,
            languages,
        })
    }

    // ── Rate-limit helpers ──────────────────────────────────────────────

    /// Public read‑only accessor for the last known `x-ratelimit-remaining` value.
    pub fn rate_limit_remaining(&self) -> Option<u32> {
        *self
            .rate_limit_remaining
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// Public read‑only accessor for the last known `x-ratelimit-reset` (Unix timestamp).
    pub fn rate_limit_reset(&self) -> Option<u64> {
        *self
            .rate_limit_reset
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    /// Extract rate‑limit headers from an HTTP response and cache them on `self`.
    pub(crate) fn update_rate_limit_from_response(&self, response: &Response) {
        if let Some(remaining) = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            && let Ok(mut guard) = self.rate_limit_remaining.lock()
        {
            *guard = Some(remaining);
        }
        if let Some(reset) = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            && let Ok(mut guard) = self.rate_limit_reset.lock()
        {
            *guard = Some(reset);
        }
    }

    /// Send a request, update rate limits, check for errors, and parse JSON response.
    /// Handles the common success case. Returns the deserialized response.
    async fn send_and_check_json<T: serde::de::DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, GitHubError> {
        let response = request.send().await.map_err(GitHubError::Network)?;
        self.update_rate_limit_from_response(&response);
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let bytes = response.bytes().await.map_err(GitHubError::Network)?;
        let data: T = serde_json::from_slice(&bytes).map_err(GitHubError::Parse)?;
        Ok(data)
    }

    /// Return an error immediately if we already know the rate limit is exhausted.
    pub(crate) fn check_rate_limit(&self) -> Result<(), GitHubError> {
        let remaining = self
            .rate_limit_remaining
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(0) = *remaining {
            let reset = self
                .rate_limit_reset
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(reset_ts) = *reset {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if reset_ts > now {
                    return Err(GitHubError::RateLimited {
                        remaining: 0,
                        reset: reset_ts,
                    });
                }
            }
            return Err(GitHubError::RateLimited {
                remaining: 0,
                reset: 0,
            });
        }
        Ok(())
    }

    pub fn new(token: Option<&str>) -> Result<Self, GitHubError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gitnapse/0.1"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
            let value =
                HeaderValue::from_str(&format!("Bearer {}", token.trim())).map_err(|e| {
                    GitHubError::Other(format!("Invalid token value for HTTP header: {e}"))
                })?;
            headers.insert(AUTHORIZATION, value);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            client,
            rate_limit_remaining: Mutex::new(None),
            rate_limit_reset: Mutex::new(None),
        })
    }

    pub fn get_runtime() -> &'static tokio::runtime::Runtime {
        runtime::get_runtime()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_me_query tests ────────────────────────────────────────────

    #[test]
    fn test_parse_me_exact() {
        let q = GitHubClient::parse_me_query("@me");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert!(q.languages.is_empty());
    }

    #[test]
    fn test_parse_me_case_insensitive() {
        let q = GitHubClient::parse_me_query("@Me");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
    }

    #[test]
    fn test_parse_me_with_terms() {
        let q = GitHubClient::parse_me_query("@me rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["rust"]);
        assert!(q.languages.is_empty());
    }

    #[test]
    fn test_parse_me_multiple_spaces() {
        let q = GitHubClient::parse_me_query("@me   rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["rust"]);
    }

    #[test]
    fn test_parse_me_with_language() {
        let q = GitHubClient::parse_me_query("@me language:rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert_eq!(q.languages, vec!["rust"]);
    }

    #[test]
    fn test_parse_me_comma_rejected() {
        assert!(GitHubClient::parse_me_query("@me,rust").is_none());
        assert!(GitHubClient::parse_me_query("@me,").is_none());
    }

    #[test]
    fn test_parse_me_special_chars() {
        let q = GitHubClient::parse_me_query("@me foo/bar");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["foo/bar"]);
    }

    #[test]
    fn test_parse_me_exact_me_colon() {
        let q = GitHubClient::parse_me_query("me:");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert!(q.languages.is_empty());
    }

    #[test]
    fn test_parse_me_me_colon_with_terms() {
        let q = GitHubClient::parse_me_query("me:rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["rust"]);
    }

    #[test]
    fn test_parse_me_me_colon_multiple_languages() {
        let q = GitHubClient::parse_me_query("me: language:rust,go");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert_eq!(q.languages, vec!["rust", "go"]);
    }

    #[test]
    fn test_parse_me_not_triggered() {
        // Not a real @me query
        assert!(GitHubClient::parse_me_query("search term").is_none());
        assert!(GitHubClient::parse_me_query("@mememe").is_none());
        assert!(GitHubClient::parse_me_query("@").is_none());
        assert!(GitHubClient::parse_me_query("").is_none());
    }
}

// ── Integration tests (mocked HTTP) ─────────────────────────────────────
#[cfg(test)]
mod integration_tests {
    use super::*;
    use mockito::{Matcher, Server};
    use serial_test::serial;

    fn with_api_base<T>(base: &str, test: impl FnOnce() -> T) -> T {
        temp_env::with_var("GITNAPSE_GITHUB_API", Some(base), test)
    }

    #[test]
    #[serial]
    fn search_general_uses_search_endpoint() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/search/repositories")
            .match_query(Matcher::Regex(
                r"q=rust\+language:rust.*per_page=30.*page=1".to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                  "items": [
                    {
                      "name": "repo-one",
                      "full_name": "x/repo-one",
                      "description": "General search result",
                      "stargazers_count": 10,
                      "language": "Rust",
                      "clone_url": "https://github.com/x/repo-one.git",
                      "owner": { "login": "x" },
                      "default_branch": "main"
                    }
                  ]
                }"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(None).expect("client");
            let repos = client
                .search_repositories_page("rust language:rust", 1, 30)
                .expect("search");
            assert_eq!(repos.len(), 1);
            assert_eq!(repos[0].full_name, "x/repo-one");
        });
    }

    #[test]
    #[serial]
    fn me_query_lists_and_filters_authenticated_repos() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/user/repos")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("visibility".into(), "all".into()),
                Matcher::UrlEncoded(
                    "affiliation".into(),
                    "owner,collaborator,organization_member".into(),
                ),
                Matcher::UrlEncoded("per_page".into(), "30".into()),
                Matcher::UrlEncoded("page".into(), "1".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                  {
                    "name": "alpha-rust",
                    "full_name": "me/alpha-rust",
                    "description": "Rust private project",
                    "stargazers_count": 1,
                    "language": "Rust",
                    "clone_url": "https://github.com/me/alpha-rust.git",
                    "owner": { "login": "me" },
                    "default_branch": "main"
                  },
                  {
                    "name": "beta-js",
                    "full_name": "me/beta-js",
                    "description": "JavaScript project",
                    "stargazers_count": 2,
                    "language": "JavaScript",
                    "clone_url": "https://github.com/me/beta-js.git",
                    "owner": { "login": "me" },
                    "default_branch": "main"
                  }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let repos = client
                .search_repositories_page("@me language:rust private", 1, 30)
                .expect("search");
            assert_eq!(repos.len(), 1);
            assert_eq!(repos[0].full_name, "me/alpha-rust");
        });
    }

    #[test]
    #[serial]
    fn me_query_returns_error_on_unauthorized() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/user/repos")
            .match_query(Matcher::Any)
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message":"Bad credentials"}"#)
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(None).expect("client");
            let err = client
                .search_repositories_page("@me", 1, 30)
                .expect_err("must fail");
            assert!(
                err.to_string().contains("Authentication required"),
                "unexpected error: {err}"
            );
        });
    }
}
