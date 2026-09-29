use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{Issue, IssueComment};

impl GitHubClient {
    /// Fetch a single issue (or pull request) by number.
    pub fn fetch_issue_detail(&self, full_name: &str, number: u64) -> Result<Issue, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_issue_detail(full_name, number))
    }

    async fn async_fetch_issue_detail(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Issue, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/issues/{number}");
            self.send_and_check_json(self.client.get(url)).await
        })
        .await
    }

    /// Fetch the conversation comments of an issue or pull request.
    ///
    /// GitHub caps `per_page` at 100; this fetches the first page.
    pub fn fetch_issue_comments(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<IssueComment>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_issue_comments(full_name, number))
    }

    pub(crate) async fn async_fetch_issue_comments(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Vec<IssueComment>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/issues/{number}/comments?per_page=100");
            self.send_and_check_json(self.client.get(url)).await
        })
        .await
    }

    /// Create a comment on an issue or pull request.
    pub fn create_issue_comment(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
    ) -> Result<IssueComment, GitHubError> {
        let full_name = full_name.to_string();
        let body = body.to_string();
        Self::get_runtime().block_on(self.async_create_issue_comment(full_name, number, body))
    }

    async fn async_create_issue_comment(
        &self,
        full_name: String,
        number: u64,
        body: String,
    ) -> Result<IssueComment, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/issues/{number}/comments");

            let payload = serde_json::json!({ "body": body });
            let response = self.client.post(url).json(&payload).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }

            let data: IssueComment = response.json().await?;
            Ok(data)
        })
        .await
    }

    /// Reopen a closed issue (`state=open`), returning the updated issue.
    pub fn reopen_issue(&self, full_name: &str, number: u64) -> Result<Issue, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_reopen_issue(full_name, number))
    }

    async fn async_reopen_issue(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Issue, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/issues/{number}");

            let payload = serde_json::json!({ "state": "open" });
            let response = self.client.patch(url).json(&payload).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }

            let data: Issue = response.json().await?;
            Ok(data)
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

    const ISSUE_JSON: &str = r#"{
        "number": 7,
        "title": "Fix the thing",
        "state": "open",
        "html_url": "https://github.com/o/r/issues/7",
        "user": { "login": "octocat", "avatar_url": "https://avatars.example/octocat.png" },
        "labels": [{ "name": "bug", "color": "ff0000" }],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-02T00:00:00Z",
        "body": "it is broken",
        "pull_request": null
    }"#;

    #[test]
    #[serial]
    fn fetch_issue_detail_parses_issue() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/repos/o/r/issues/7")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ISSUE_JSON)
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let issue = client.fetch_issue_detail("o/r", 7).expect("issue");
            assert_eq!(issue.number, 7);
            assert_eq!(issue.title, "Fix the thing");
            assert_eq!(issue.user.login, "octocat");
            assert_eq!(
                issue.user.avatar_url.as_deref(),
                Some("https://avatars.example/octocat.png")
            );
            assert_eq!(issue.labels.len(), 1);
        });
    }

    #[test]
    #[serial]
    fn fetch_issue_comments_parses_list() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/repos/o/r/issues/7/comments")
            .match_query(Matcher::UrlEncoded("per_page".into(), "100".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "id": 42,
                        "user": { "login": "hubot", "avatar_url": null },
                        "body": "first",
                        "created_at": "2026-01-03T00:00:00Z",
                        "updated_at": "2026-01-03T00:00:00Z",
                        "html_url": "https://github.com/o/r/issues/7#issuecomment-42"
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(None).expect("client");
            let comments = client.fetch_issue_comments("o/r", 7).expect("comments");
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].id, 42);
            assert_eq!(comments[0].body, "first");
            assert_eq!(comments[0].user.login, "hubot");
            assert!(comments[0].user.avatar_url.is_none());
        });
    }

    #[test]
    #[serial]
    fn create_issue_comment_posts_body() {
        let mut server = Server::new();
        let _m = server
            .mock("POST", "/repos/o/r/issues/7/comments")
            .match_body(Matcher::Json(serde_json::json!({ "body": "hello" })))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": 43,
                    "user": { "login": "me", "avatar_url": "https://avatars.example/me.png" },
                    "body": "hello",
                    "created_at": "2026-01-04T00:00:00Z",
                    "updated_at": "2026-01-04T00:00:00Z",
                    "html_url": "https://github.com/o/r/issues/7#issuecomment-43"
                }"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let comment = client
                .create_issue_comment("o/r", 7, "hello")
                .expect("comment");
            assert_eq!(comment.id, 43);
            assert_eq!(comment.body, "hello");
            assert_eq!(comment.user.login, "me");
        });
    }

    #[test]
    #[serial]
    fn reopen_issue_patches_state_open() {
        let mut server = Server::new();
        let _m = server
            .mock("PATCH", "/repos/o/r/issues/7")
            .match_body(Matcher::Json(serde_json::json!({ "state": "open" })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ISSUE_JSON)
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let issue = client.reopen_issue("o/r", 7).expect("reopen");
            assert_eq!(issue.state, "open");
        });
    }
}
