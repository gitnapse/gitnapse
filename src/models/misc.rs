use serde::Deserialize;

use super::IssueUser;

/// A commit in a repository
#[derive(Debug, Clone, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub commit: CommitDetails,
    /// GitHub user associated with the commit, when GitHub can match the git
    /// author to an account (the raw git author stays in `commit.author`).
    #[serde(default)]
    pub author: Option<IssueUser>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitDetails {
    pub message: String,
    pub author: CommitAuthor,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub date: String,
}

/// A comparison between two commits (for diffs)
#[derive(Debug, Deserialize)]
pub struct CompareResponse {
    pub status: String,
    pub ahead_by: u32,
    pub behind_by: u32,
    pub total_commits: u32,
    pub files: Vec<DiffFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiffFile {
    pub filename: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub changes: u32,
    pub patch: Option<String>,
}

/// CI check run
#[derive(Debug, Deserialize)]
pub struct CheckRunsResponse {
    pub check_runs: Vec<CheckRun>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Workflow run
#[derive(Debug, Deserialize)]
pub struct WorkflowRunsResponse {
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}
