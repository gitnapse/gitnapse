use serde::{Deserialize, Serialize};

// ── Issue-related ─────────────────────────────────────────────────────

/// A GitHub Issue
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub user: IssueUser,
    pub labels: Vec<IssueLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub body: Option<String>,
    pub pull_request: Option<PrInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IssueUser {
    pub login: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueLabel {
    pub name: String,
    pub color: String,
}

/// Minimal PR info (present in issue if it's a PR)
#[derive(Debug, Clone, Deserialize)]
pub struct PrInfo {
    pub url: Option<String>,
    pub html_url: Option<String>,
}

// ── PR-related ────────────────────────────────────────────────────────

/// A Pull Request
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub user: IssueUser,
    pub body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changed_files: Option<u32>,
}

/// A review on a pull request
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestReview {
    pub id: u64,
    pub user: IssueUser,
    pub body: Option<String>,
    pub state: String,
    pub submitted_at: Option<String>,
    pub commit_id: Option<String>,
}

/// A review comment on a pull request (inline)
#[derive(Debug, Clone, Deserialize)]
pub struct ReviewComment {
    pub id: u64,
    pub user: IssueUser,
    pub body: String,
    pub path: Option<String>,
    pub position: Option<u64>,
    pub commit_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Response from merging a pull request
#[derive(Debug, Clone, Deserialize)]
pub struct MergeResponse {
    pub sha: String,
    pub merged: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePrRequest {
    pub title: String,
    pub head: String,
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// Full pull request detail
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestDetail {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub body: Option<String>,
    pub html_url: String,
    pub user: IssueUser,
    pub created_at: String,
    pub updated_at: String,
    pub merge_commit_sha: Option<String>,
    pub merged: Option<bool>,
    pub merged_by: Option<IssueUser>,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changed_files: Option<u32>,
    pub commits: Option<u32>,
    pub comments: Option<u32>,
    pub review_comments: Option<u32>,
    pub head: PrBranch,
    pub base: PrBranch,
    pub labels: Vec<IssueLabel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrBranch {
    pub label: String,
    pub r#ref: String,
    pub sha: String,
}
