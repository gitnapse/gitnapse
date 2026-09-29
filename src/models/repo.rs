use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RepoSummary {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub language: Option<String>,
    pub clone_url: String,
    pub owner: RepoOwner,
    pub default_branch: String,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub forks_count: Option<u64>,
    #[serde(default)]
    pub open_issues_count: Option<u64>,
    #[serde(default)]
    pub watchers_count: Option<u64>,
    #[serde(default)]
    pub private: Option<bool>,
    #[serde(default)]
    pub topics: Option<Vec<String>>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub pushed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoOwner {
    pub login: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<RepoSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BranchInfo {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct TreeResponse {
    pub tree: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub sha: String,
}

#[derive(Debug, Clone)]
pub struct RepoNode {
    pub path: String,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentResponse {
    pub sha: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthenticatedUser {
    pub login: String,
}
