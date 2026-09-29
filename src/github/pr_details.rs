use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{DiffFile, IssueComment};

impl GitHubClient {
    /// Fetch the files changed by a pull request (first page, up to 100).
    pub fn fetch_pr_files(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<DiffFile>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_pr_files(full_name, number))
    }

    async fn async_fetch_pr_files(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Vec<DiffFile>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/files?per_page=100");
            self.send_and_check_json(self.client.get(url)).await
        })
        .await
    }

    /// Fetch the conversation comments of a pull request.
    ///
    /// Pull request conversations use the issue comments endpoint.
    pub fn fetch_pr_conversation(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<IssueComment>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_issue_comments(full_name, number))
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

    #[test]
    #[serial]
    fn fetch_pr_files_parses_changed_files() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/repos/o/r/pulls/3/files")
            .match_query(Matcher::UrlEncoded("per_page".into(), "100".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "filename": "src/main.rs",
                        "status": "modified",
                        "additions": 10,
                        "deletions": 2,
                        "changes": 12,
                        "patch": "@@ -1 +1 @@\n-old\n+new"
                    },
                    {
                        "filename": "README.md",
                        "status": "added",
                        "additions": 3,
                        "deletions": 0,
                        "changes": 3,
                        "patch": null
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let files = client.fetch_pr_files("o/r", 3).expect("files");
            assert_eq!(files.len(), 2);
            assert_eq!(files[0].filename, "src/main.rs");
            assert_eq!(files[0].status, "modified");
            assert_eq!(files[0].additions, 10);
            assert!(files[0].patch.is_some());
            assert_eq!(files[1].filename, "README.md");
            assert!(files[1].patch.is_none());
        });
    }

    #[test]
    #[serial]
    fn fetch_pr_conversation_uses_issue_comments_endpoint() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/repos/o/r/issues/3/comments")
            .match_query(Matcher::UrlEncoded("per_page".into(), "100".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "id": 9,
                        "user": { "login": "reviewer", "avatar_url": null },
                        "body": "looks good",
                        "created_at": "2026-01-12T00:00:00Z",
                        "updated_at": "2026-01-12T00:00:00Z",
                        "html_url": "https://github.com/o/r/pull/3#issuecomment-9"
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let comments = client
                .fetch_pr_conversation("o/r", 3)
                .expect("conversation");
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].body, "looks good");
            assert_eq!(comments[0].user.login, "reviewer");
        });
    }
}
