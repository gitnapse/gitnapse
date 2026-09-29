use crate::error::GitHubError;
use crate::github::{GitHubClient, encode_query_value, with_retry};
use crate::models::{Notification, RepoSummary, UserEvent, UserProfile};
use serde::Deserialize;

// ── Raw API payloads ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SearchUsersResponse {
    items: Vec<UserProfile>,
}

#[derive(Debug, Default, Deserialize)]
struct RawUserEvent {
    #[serde(default)]
    id: String,
    #[serde(rename = "type", default)]
    event_type: String,
    #[serde(default)]
    actor: RawEventActor,
    #[serde(default)]
    repo: RawEventRepo,
    #[serde(default)]
    payload: serde_json::Value,
    #[serde(default)]
    created_at: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawEventActor {
    #[serde(default)]
    login: String,
    #[serde(default)]
    avatar_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawEventRepo {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawNotification {
    #[serde(default)]
    id: String,
    #[serde(default)]
    unread: bool,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    subject: RawNotificationSubject,
    #[serde(default)]
    repository: Option<RawNotificationRepo>,
    #[serde(default)]
    updated_at: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawNotificationSubject {
    #[serde(default)]
    title: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawNotificationRepo {
    #[serde(default)]
    full_name: Option<String>,
}

// ── Event mapping (pure, unit-tested) ────────────────────────────────

fn event_kind(event_type: &str) -> &'static str {
    match event_type {
        "PushEvent" => "push",
        "PullRequestEvent" => "pull_request",
        "IssuesEvent" => "issues",
        "ReleaseEvent" => "release",
        "CreateEvent" => "create",
        "WatchEvent" => "watch",
        "ForkEvent" => "fork",
        _ => "other",
    }
}

fn short_ref(git_ref: &str) -> &str {
    git_ref
        .strip_prefix("refs/heads/")
        .or_else(|| git_ref.strip_prefix("refs/tags/"))
        .unwrap_or(git_ref)
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn commit_count(count: u64) -> String {
    if count == 1 {
        "1 commit".to_string()
    } else {
        format!("{count} commits")
    }
}

/// Human summary for a known event payload, `None` when the payload does not
/// carry enough information (or the event type is unknown).
fn event_title(event_type: &str, payload: &serde_json::Value, repo: &str) -> Option<String> {
    match event_type {
        "PushEvent" => {
            let count = payload
                .get("size")
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    payload
                        .get("commits")
                        .and_then(serde_json::Value::as_array)
                        .map(|commits| commits.len() as u64)
                });
            let branch = payload
                .get("ref")
                .and_then(serde_json::Value::as_str)
                .map(short_ref)
                .filter(|branch| !branch.is_empty());
            match (count, branch) {
                (Some(count), Some(branch)) => {
                    Some(format!("Pushed {} to {branch}", commit_count(count)))
                }
                (Some(count), None) => Some(format!("Pushed {}", commit_count(count))),
                (None, Some(branch)) => Some(format!("Pushed commits to {branch}")),
                (None, None) => None,
            }
        }
        "PullRequestEvent" => {
            let pr = payload.get("pull_request")?;
            let number = pr.get("number").and_then(serde_json::Value::as_u64)?;
            let title = pr.get("title").and_then(serde_json::Value::as_str)?;
            let action = payload
                .get("action")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("updated");
            Some(format!("{} PR #{number}: {title}", capitalize(action)))
        }
        "IssuesEvent" => {
            let issue = payload.get("issue")?;
            let number = issue.get("number").and_then(serde_json::Value::as_u64)?;
            let title = issue.get("title").and_then(serde_json::Value::as_str)?;
            let action = payload
                .get("action")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("updated");
            Some(format!("{} issue #{number}: {title}", capitalize(action)))
        }
        "ReleaseEvent" => {
            let release = payload.get("release")?;
            let tag = release
                .get("tag_name")
                .and_then(serde_json::Value::as_str)
                .or_else(|| release.get("name").and_then(serde_json::Value::as_str))?;
            Some(format!("Released {tag}"))
        }
        "CreateEvent" => {
            let ref_type = payload
                .get("ref_type")
                .and_then(serde_json::Value::as_str)?;
            match payload
                .get("ref")
                .and_then(serde_json::Value::as_str)
                .filter(|name| !name.is_empty())
            {
                Some(name) => Some(format!("Created {ref_type} {name}")),
                None => Some(format!("Created {ref_type}")),
            }
        }
        "WatchEvent" if !repo.is_empty() => Some(format!("Starred {repo}")),
        "ForkEvent" if !repo.is_empty() => Some(format!("Forked {repo}")),
        _ => None,
    }
}

fn event_action(event_type: &str, payload: &serde_json::Value) -> Option<String> {
    if let Some(action) = payload.get("action").and_then(serde_json::Value::as_str) {
        return Some(action.to_string());
    }
    match event_type {
        "WatchEvent" => Some("started".to_string()),
        "ForkEvent" => Some("forked".to_string()),
        _ => None,
    }
}

fn map_user_event(raw: RawUserEvent) -> UserEvent {
    let title = event_title(&raw.event_type, &raw.payload, &raw.repo.name);
    UserEvent {
        id: raw.id,
        kind: event_kind(&raw.event_type).to_string(),
        actor: raw.actor.login,
        actor_avatar_url: raw.actor.avatar_url,
        repo: raw.repo.name,
        action: event_action(&raw.event_type, &raw.payload),
        title,
        created_at: raw.created_at,
    }
}

/// Convert an `api.github.com` URL into its `github.com` web counterpart.
///
/// API paths carry a `/repos/` segment that is not part of the web URL, e.g.
/// `https://api.github.com/repos/o/r/issues/7` -> `https://github.com/o/r/issues/7`.
fn api_url_to_html(url: &str) -> Option<String> {
    let path = url
        .strip_prefix("https://api.github.com/")
        .or_else(|| url.strip_prefix("http://api.github.com/"))?;
    let web_path = path.strip_prefix("repos/").unwrap_or(path);
    Some(format!("https://github.com/{web_path}"))
}

fn map_notification(raw: RawNotification) -> Notification {
    Notification {
        id: raw.id,
        unread: raw.unread,
        reason: raw.reason,
        subject_type: raw.subject.kind,
        subject_title: raw.subject.title,
        repo: raw.repository.and_then(|repo| repo.full_name),
        updated_at: raw.updated_at,
        html_url: raw.subject.url.as_deref().and_then(api_url_to_html),
    }
}

// ── Client methods ───────────────────────────────────────────────────

impl GitHubClient {
    /// Fetch a user's public profile.
    pub fn fetch_user_profile(&self, login: &str) -> Result<UserProfile, GitHubError> {
        let login = login.to_string();
        Self::get_runtime().block_on(self.async_fetch_user_profile(login))
    }

    async fn async_fetch_user_profile(&self, login: String) -> Result<UserProfile, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/users/{login}");
            self.send_and_check_json(self.client.get(url)).await
        })
        .await
    }

    /// List a user's public repositories.
    ///
    /// `sort` is one of `created`, `updated`, `pushed` or `full_name`; anything
    /// else falls back to `updated`.
    pub fn fetch_user_repos(
        &self,
        login: &str,
        sort: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        let login = login.to_string();
        let sort = sort.to_string();
        Self::get_runtime().block_on(self.async_fetch_user_repos(login, sort, page, per_page))
    }

    async fn async_fetch_user_repos(
        &self,
        login: String,
        sort: String,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let sort = match sort.trim() {
                "created" | "updated" | "pushed" | "full_name" => sort.trim(),
                _ => "updated",
            };
            let page = page.max(1);
            let per_page = per_page.clamp(1, 100);
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/users/{login}/repos?sort={sort}&page={page}&per_page={per_page}"
            );
            self.send_and_check_json(self.client.get(url)).await
        })
        .await
    }

    /// Search GitHub users.
    pub fn search_users(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<UserProfile>, GitHubError> {
        let query = query.to_string();
        Self::get_runtime().block_on(self.async_search_users(query, page, per_page))
    }

    async fn async_search_users(
        &self,
        query: String,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<UserProfile>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let page = page.max(1);
            let per_page = per_page.clamp(1, 100);
            let api_base = Self::api_base();
            let query = encode_query_value(query.trim());
            let url = format!("{api_base}/search/users?q={query}&page={page}&per_page={per_page}");
            let data: SearchUsersResponse = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data.items)
        })
        .await
    }

    /// Fetch a user's public activity feed as compact display events.
    pub fn fetch_user_events(
        &self,
        login: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<UserEvent>, GitHubError> {
        let login = login.to_string();
        Self::get_runtime().block_on(self.async_fetch_user_events(login, page, per_page))
    }

    async fn async_fetch_user_events(
        &self,
        login: String,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<UserEvent>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let page = page.max(1);
            let per_page = per_page.clamp(1, 100);
            let api_base = Self::api_base();
            let url =
                format!("{api_base}/users/{login}/events/public?page={page}&per_page={per_page}");
            let raw: Vec<RawUserEvent> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(raw.into_iter().map(map_user_event).collect())
        })
        .await
    }

    /// Fetch the authenticated user's unread notifications.
    pub fn fetch_notifications(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<Notification>, GitHubError> {
        Self::get_runtime().block_on(self.async_fetch_notifications(page, per_page))
    }

    async fn async_fetch_notifications(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<Notification>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let page = page.max(1);
            let per_page = per_page.clamp(1, 100);
            let api_base = Self::api_base();
            let url = format!("{api_base}/notifications?all=false&page={page}&per_page={per_page}");
            let raw: Vec<RawNotification> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(raw.into_iter().map(map_notification).collect())
        })
        .await
    }

    /// Mark a notification thread as read.
    pub fn mark_notification_read(&self, id: &str) -> Result<(), GitHubError> {
        let id = id.to_string();
        Self::get_runtime().block_on(self.async_mark_notification_read(id))
    }

    async fn async_mark_notification_read(&self, id: String) -> Result<(), GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/notifications/threads/{id}");
            let response = self.client.patch(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};
    use serial_test::serial;

    fn with_api_base<T>(base: &str, test: impl FnOnce() -> T) -> T {
        temp_env::with_var("GITNAPSE_GITHUB_API", Some(base), test)
    }

    // ── Event mapping unit tests (no HTTP) ───────────────────────────

    #[test]
    fn event_kind_maps_known_and_unknown_types() {
        assert_eq!(event_kind("PushEvent"), "push");
        assert_eq!(event_kind("PullRequestEvent"), "pull_request");
        assert_eq!(event_kind("IssuesEvent"), "issues");
        assert_eq!(event_kind("ReleaseEvent"), "release");
        assert_eq!(event_kind("CreateEvent"), "create");
        assert_eq!(event_kind("WatchEvent"), "watch");
        assert_eq!(event_kind("ForkEvent"), "fork");
        assert_eq!(event_kind("MysteryEvent"), "other");
    }

    #[test]
    fn event_title_formats_known_payloads() {
        let payload = serde_json::json!({
            "ref": "refs/heads/main",
            "size": 3,
        });
        assert_eq!(
            event_title("PushEvent", &payload, "o/r").as_deref(),
            Some("Pushed 3 commits to main")
        );

        let payload = serde_json::json!({
            "ref": "refs/heads/main",
            "size": 1,
        });
        assert_eq!(
            event_title("PushEvent", &payload, "o/r").as_deref(),
            Some("Pushed 1 commit to main")
        );

        let payload = serde_json::json!({
            "action": "opened",
            "pull_request": { "number": 12, "title": "Add feature" },
        });
        assert_eq!(
            event_title("PullRequestEvent", &payload, "o/r").as_deref(),
            Some("Opened PR #12: Add feature")
        );

        let payload = serde_json::json!({ "release": { "tag_name": "v1.0.0" } });
        assert_eq!(
            event_title("ReleaseEvent", &payload, "o/r").as_deref(),
            Some("Released v1.0.0")
        );

        let payload = serde_json::json!({ "ref_type": "branch", "ref": "dev" });
        assert_eq!(
            event_title("CreateEvent", &payload, "o/r").as_deref(),
            Some("Created branch dev")
        );

        assert_eq!(
            event_title("WatchEvent", &serde_json::json!({}), "o/r").as_deref(),
            Some("Starred o/r")
        );
        assert_eq!(
            event_title("ForkEvent", &serde_json::json!({}), "o/r").as_deref(),
            Some("Forked o/r")
        );
    }

    #[test]
    fn event_title_returns_none_for_unknown_or_empty_payloads() {
        assert_eq!(
            event_title("MysteryEvent", &serde_json::json!({ "x": 1 }), "o/r"),
            None
        );
        assert_eq!(
            event_title("PushEvent", &serde_json::json!({}), "o/r"),
            None
        );
        assert_eq!(
            event_title(
                "PullRequestEvent",
                &serde_json::json!({ "action": "opened" }),
                "o/r"
            ),
            None
        );
    }

    #[test]
    fn map_notification_converts_api_url_to_html() {
        let raw = RawNotification {
            id: "9".into(),
            unread: true,
            reason: "mention".into(),
            subject: RawNotificationSubject {
                title: "A thread".into(),
                kind: "Issue".into(),
                url: Some("https://api.github.com/repos/o/r/issues/7".into()),
            },
            repository: Some(RawNotificationRepo {
                full_name: Some("o/r".into()),
            }),
            updated_at: "2026-01-08T00:00:00Z".into(),
        };
        let notification = map_notification(raw);
        assert_eq!(notification.id, "9");
        assert!(notification.unread);
        assert_eq!(notification.subject_type, "Issue");
        assert_eq!(notification.subject_title, "A thread");
        assert_eq!(notification.repo.as_deref(), Some("o/r"));
        assert_eq!(
            notification.html_url.as_deref(),
            Some("https://github.com/o/r/issues/7")
        );
    }

    #[test]
    fn api_url_to_html_handles_repos_and_other_paths() {
        assert_eq!(
            api_url_to_html("https://api.github.com/repos/o/r/pulls/3").as_deref(),
            Some("https://github.com/o/r/pulls/3")
        );
        assert_eq!(
            api_url_to_html("https://api.github.com/notifications/threads/1").as_deref(),
            Some("https://github.com/notifications/threads/1")
        );
        assert_eq!(api_url_to_html("https://github.com/o/r"), None);
    }

    // ── HTTP tests ───────────────────────────────────────────────────

    #[test]
    #[serial]
    fn fetch_user_profile_parses_profile() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/users/octocat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "login": "octocat",
                    "name": "The Octocat",
                    "avatar_url": "https://avatars.example/octocat.png",
                    "bio": "there once was...",
                    "company": "@github",
                    "location": "San Francisco",
                    "blog": "https://example.com",
                    "followers": 100,
                    "following": 9,
                    "public_repos": 8,
                    "html_url": "https://github.com/octocat",
                    "created_at": "2011-01-25T18:44:36Z"
                }"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let profile = client.fetch_user_profile("octocat").expect("profile");
            assert_eq!(profile.login, "octocat");
            assert_eq!(profile.name.as_deref(), Some("The Octocat"));
            assert_eq!(profile.followers, 100);
            assert_eq!(profile.following, 9);
            assert_eq!(profile.public_repos, 8);
            assert_eq!(profile.blog.as_deref(), Some("https://example.com"));
        });
    }

    #[test]
    #[serial]
    fn fetch_user_repos_uses_sort_and_pagination() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/users/octocat/repos")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("sort".into(), "pushed".into()),
                Matcher::UrlEncoded("page".into(), "2".into()),
                Matcher::UrlEncoded("per_page".into(), "5".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "name": "repo-one",
                        "full_name": "octocat/repo-one",
                        "description": "A repo",
                        "stargazers_count": 10,
                        "language": "Rust",
                        "clone_url": "https://github.com/octocat/repo-one.git",
                        "owner": { "login": "octocat", "avatar_url": "https://avatars.example/octocat.png" },
                        "default_branch": "main",
                        "html_url": "https://github.com/octocat/repo-one",
                        "forks_count": 4,
                        "open_issues_count": 2,
                        "watchers_count": 10,
                        "private": false,
                        "topics": ["cli", "rust"],
                        "updated_at": "2026-01-09T00:00:00Z",
                        "pushed_at": "2026-01-10T00:00:00Z"
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let repos = client
                .fetch_user_repos("octocat", "pushed", 2, 5)
                .expect("repos");
            assert_eq!(repos.len(), 1);
            let repo = &repos[0];
            assert_eq!(repo.full_name, "octocat/repo-one");
            assert_eq!(
                repo.html_url.as_deref(),
                Some("https://github.com/octocat/repo-one")
            );
            assert_eq!(repo.forks_count, Some(4));
            assert_eq!(repo.open_issues_count, Some(2));
            assert_eq!(repo.private, Some(false));
            assert_eq!(
                repo.topics,
                Some(vec!["cli".to_string(), "rust".to_string()])
            );
            assert_eq!(
                repo.owner.avatar_url.as_deref(),
                Some("https://avatars.example/octocat.png")
            );
        });
    }

    #[test]
    #[serial]
    fn search_users_maps_reduced_user_objects() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/search/users")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "octo cat".into()),
                Matcher::UrlEncoded("page".into(), "1".into()),
                Matcher::UrlEncoded("per_page".into(), "10".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "total_count": 1,
                    "items": [
                        {
                            "login": "octocat",
                            "avatar_url": "https://avatars.example/octocat.png",
                            "html_url": "https://github.com/octocat"
                        }
                    ]
                }"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let users = client.search_users("octo cat", 1, 10).expect("users");
            assert_eq!(users.len(), 1);
            assert_eq!(users[0].login, "octocat");
            // Reduced search objects omit the counters; they default to 0.
            assert_eq!(users[0].followers, 0);
            assert_eq!(users[0].following, 0);
            assert_eq!(users[0].public_repos, 0);
        });
    }

    #[test]
    #[serial]
    fn fetch_user_events_maps_display_events() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/users/octocat/events/public")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("page".into(), "1".into()),
                Matcher::UrlEncoded("per_page".into(), "30".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "id": "1",
                        "type": "PushEvent",
                        "actor": { "login": "octocat", "avatar_url": "https://avatars.example/octocat.png" },
                        "repo": { "name": "o/r" },
                        "payload": { "ref": "refs/heads/main", "size": 3, "commits": [] },
                        "created_at": "2026-01-05T00:00:00Z"
                    },
                    {
                        "id": "2",
                        "type": "WatchEvent",
                        "actor": { "login": "octocat" },
                        "repo": { "name": "o/r" },
                        "payload": { "action": "started" },
                        "created_at": "2026-01-06T00:00:00Z"
                    },
                    {
                        "id": "3",
                        "type": "SomethingNewEvent",
                        "actor": { "login": "octocat" },
                        "repo": { "name": "o/r" },
                        "payload": { "weird": true },
                        "created_at": "2026-01-07T00:00:00Z"
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let events = client.fetch_user_events("octocat", 1, 30).expect("events");
            assert_eq!(events.len(), 3);

            assert_eq!(events[0].kind, "push");
            assert_eq!(events[0].actor, "octocat");
            assert_eq!(events[0].repo, "o/r");
            assert_eq!(events[0].title.as_deref(), Some("Pushed 3 commits to main"));
            assert_eq!(
                events[0].actor_avatar_url.as_deref(),
                Some("https://avatars.example/octocat.png")
            );

            assert_eq!(events[1].kind, "watch");
            assert_eq!(events[1].action.as_deref(), Some("started"));
            assert_eq!(events[1].title.as_deref(), Some("Starred o/r"));

            assert_eq!(events[2].kind, "other");
            assert!(events[2].title.is_none());
        });
    }

    #[test]
    #[serial]
    fn fetch_notifications_maps_threads() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/notifications")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("all".into(), "false".into()),
                Matcher::UrlEncoded("page".into(), "1".into()),
                Matcher::UrlEncoded("per_page".into(), "50".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "id": "77",
                        "unread": true,
                        "reason": "review_requested",
                        "updated_at": "2026-01-11T00:00:00Z",
                        "subject": {
                            "title": "Improve docs",
                            "type": "PullRequest",
                            "url": "https://api.github.com/repos/o/r/pulls/3"
                        },
                        "repository": { "full_name": "o/r" }
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let notifications = client.fetch_notifications(1, 50).expect("notifications");
            assert_eq!(notifications.len(), 1);
            let notification = &notifications[0];
            assert_eq!(notification.id, "77");
            assert!(notification.unread);
            assert_eq!(notification.reason, "review_requested");
            assert_eq!(notification.subject_type, "PullRequest");
            assert_eq!(notification.subject_title, "Improve docs");
            assert_eq!(notification.repo.as_deref(), Some("o/r"));
            assert_eq!(
                notification.html_url.as_deref(),
                Some("https://github.com/o/r/pulls/3")
            );
        });
    }

    #[test]
    #[serial]
    fn mark_notification_read_patches_thread() {
        let mut server = Server::new();
        let _m = server
            .mock("PATCH", "/notifications/threads/77")
            .with_status(205)
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            client.mark_notification_read("77").expect("mark read");
        });
        _m.assert();
    }
}
