use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{Contributor, LanguageStat};
use std::collections::HashMap;

impl GitHubClient {
    /// Fetch the language breakdown of a repository, sorted by bytes descending.
    pub fn fetch_languages(&self, full_name: &str) -> Result<Vec<LanguageStat>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_languages(full_name))
    }

    async fn async_fetch_languages(
        &self,
        full_name: String,
    ) -> Result<Vec<LanguageStat>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/languages");
            let bytes_by_language: HashMap<String, u64> =
                self.send_and_check_json(self.client.get(url)).await?;

            let mut stats: Vec<LanguageStat> = bytes_by_language
                .into_iter()
                .map(|(name, bytes)| LanguageStat { name, bytes })
                .collect();
            stats.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.name.cmp(&b.name)));
            Ok(stats)
        })
        .await
    }

    /// Fetch the contributors of a repository.
    pub fn fetch_contributors(
        &self,
        full_name: &str,
        per_page: u8,
    ) -> Result<Vec<Contributor>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_contributors(full_name, per_page))
    }

    async fn async_fetch_contributors(
        &self,
        full_name: String,
        per_page: u8,
    ) -> Result<Vec<Contributor>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let per_page = per_page.clamp(1, 100);
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/contributors?per_page={per_page}");
            self.send_and_check_json(self.client.get(url)).await
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
    fn fetch_languages_maps_and_sorts_by_bytes_desc() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/repos/o/r/languages")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{ "Rust": 120000, "TypeScript": 45000, "Shell": 500 }"#)
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let languages = client.fetch_languages("o/r").expect("languages");
            assert_eq!(languages.len(), 3);
            assert_eq!(languages[0].name, "Rust");
            assert_eq!(languages[0].bytes, 120000);
            assert_eq!(languages[1].name, "TypeScript");
            assert_eq!(languages[2].name, "Shell");
        });
    }

    #[test]
    #[serial]
    fn fetch_contributors_parses_list() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/repos/o/r/contributors")
            .match_query(Matcher::UrlEncoded("per_page".into(), "10".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                    {
                        "login": "octocat",
                        "avatar_url": "https://avatars.example/octocat.png",
                        "contributions": 42,
                        "html_url": "https://github.com/octocat"
                    }
                ]"#,
            )
            .create();

        with_api_base(&server.url(), || {
            let client = GitHubClient::new(Some("token")).expect("client");
            let contributors = client.fetch_contributors("o/r", 10).expect("contributors");
            assert_eq!(contributors.len(), 1);
            assert_eq!(contributors[0].login, "octocat");
            assert_eq!(contributors[0].contributions, 42);
            assert_eq!(
                contributors[0].html_url.as_deref(),
                Some("https://github.com/octocat")
            );
        });
    }
}
