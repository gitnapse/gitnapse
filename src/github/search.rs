use crate::error::GitHubError;
use crate::github::{GitHubClient, encode_query_value, with_retry};
use crate::models::CodeSearchResult;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SearchCodeResponse {
    items: Vec<RawCodeSearchItem>,
}

#[derive(Debug, Deserialize)]
struct RawCodeSearchItem {
    repository: RawCodeSearchRepo,
    path: String,
    name: String,
    sha: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct RawCodeSearchRepo {
    full_name: String,
}

impl GitHubClient {
    /// Search code across GitHub (`GET /search/code`).
    ///
    /// Requires an authenticated token: code search is not available to
    /// anonymous requests. The repository `full_name` is flattened into
    /// `CodeSearchResult::repo`.
    pub fn search_code(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<CodeSearchResult>, GitHubError> {
        let query = query.to_string();
        Self::get_runtime().block_on(self.async_search_code(query, page, per_page))
    }

    async fn async_search_code(
        &self,
        query: String,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<CodeSearchResult>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let page = page.max(1);
            let per_page = per_page.clamp(1, 100);
            let api_base = Self::api_base();
            let query = encode_query_value(query.trim());
            let url = format!("{api_base}/search/code?q={query}&page={page}&per_page={per_page}");
            let data: SearchCodeResponse = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data
                .items
                .into_iter()
                .map(|item| CodeSearchResult {
                    repo: item.repository.full_name,
                    path: item.path,
                    name: item.name,
                    sha: item.sha,
                    html_url: item.html_url,
                })
                .collect())
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

    #[test]
    #[serial]
    fn search_code_flattens_repository_full_name() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/search/code")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "add_token language:rust".into()),
                Matcher::UrlEncoded("page".into(), "1".into()),
                Matcher::UrlEncoded("per_page".into(), "20".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "total_count": 1,
                    "items": [
                        {
                            "name": "auth.rs",
                            "path": "src/auth.rs",
                            "sha": "abc123",
                            "html_url": "https://github.com/o/r/blob/main/src/auth.rs",
                            "repository": { "full_name": "o/r" }
                        }
                    ]
                }"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let results = client
                .search_code("add_token language:rust", 1, 20)
                .expect("search");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].repo, "o/r");
            assert_eq!(results[0].path, "src/auth.rs");
            assert_eq!(results[0].name, "auth.rs");
            assert_eq!(results[0].sha, "abc123");
        });
    }
}
