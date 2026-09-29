use crate::models::{
    CheckRun, CodeSearchResult, CommitInfo, CompareResponse, Contributor, DiffFile, Issue,
    IssueComment, LanguageStat, MergeResponse, Notification, PullRequest, PullRequestDetail,
    PullRequestReview, Release, RepoNode, RepoSummary, ReviewComment, UserEvent, UserProfile,
    WorkflowRun,
};
use anyhow::Result;
use std::sync::Arc;

/// The kind of git/DevOps provider detected from a remote URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    GitHub,
    AzureDevOps,
    GitLab,
    Bitbucket,
    Other,
}

impl ProviderKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::GitHub => "GitHub",
            ProviderKind::AzureDevOps => "Azure DevOps",
            ProviderKind::GitLab => "GitLab",
            ProviderKind::Bitbucket => "Bitbucket",
            ProviderKind::Other => "Other",
        }
    }
}

/// Detect the provider kind from a git remote URL.
pub fn detect_provider(remote_url: &str) -> ProviderKind {
    let url = remote_url.to_lowercase();
    if url.contains("github") {
        ProviderKind::GitHub
    } else if url.contains("dev.azure") || url.contains("visualstudio.com") {
        ProviderKind::AzureDevOps
    } else if url.contains("gitlab") {
        ProviderKind::GitLab
    } else if url.contains("bitbucket") {
        ProviderKind::Bitbucket
    } else {
        ProviderKind::Other
    }
}

/// Abstract interface for interacting with a git/DevOps hosting provider.
pub trait GitProvider: Send + Sync {
    fn fetch_authenticated_user(&self) -> Result<Option<String>>;

    fn search_repositories_page(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>>;
    fn fetch_branches(&self, full_name: &str) -> Result<Vec<String>>;
    fn fetch_repo_tree(&self, full_name: &str, branch: &str) -> Result<Vec<RepoNode>>;
    fn fetch_starred_repos(&self, page: u32, per_page: u8) -> Result<Vec<RepoSummary>>;
    fn fetch_repo_by_name(&self, full_name: &str) -> Result<RepoSummary>;

    fn fetch_user_profile(&self, login: &str) -> Result<UserProfile>;
    fn fetch_user_repos(
        &self,
        login: &str,
        sort: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>>;
    fn search_users(&self, query: &str, page: u32, per_page: u8) -> Result<Vec<UserProfile>>;
    fn fetch_user_events(&self, login: &str, page: u32, per_page: u8) -> Result<Vec<UserEvent>>;
    fn fetch_notifications(&self, page: u32, per_page: u8) -> Result<Vec<Notification>>;
    fn mark_notification_read(&self, id: &str) -> Result<()>;
    fn search_code(&self, query: &str, page: u32, per_page: u8) -> Result<Vec<CodeSearchResult>>;

    fn fetch_file_content(&self, full_name: &str, path: &str) -> Result<Vec<u8>>;
    fn fetch_file_content_by_ref(
        &self,
        full_name: &str,
        path: &str,
        git_ref: &str,
    ) -> Result<Vec<u8>>;

    fn fetch_issues(&self, full_name: &str, state: &str, per_page: u8) -> Result<Vec<Issue>>;
    fn fetch_issue_detail(&self, full_name: &str, number: u64) -> Result<Issue>;
    fn fetch_issue_comments(&self, full_name: &str, number: u64) -> Result<Vec<IssueComment>>;
    fn create_issue_comment(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
    ) -> Result<IssueComment>;
    fn reopen_issue(&self, full_name: &str, number: u64) -> Result<Issue>;
    fn create_issue(&self, full_name: &str, title: &str, body: Option<&str>) -> Result<Issue>;
    fn close_issue(&self, full_name: &str, number: u64) -> Result<Issue>;

    fn fetch_pull_requests(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<PullRequest>>;
    fn fetch_pull_request_detail(&self, full_name: &str, number: u64) -> Result<PullRequestDetail>;
    fn fetch_pull_request_reviews(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<PullRequestReview>>;
    fn fetch_pull_request_comments(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>>;
    fn fetch_pull_request_commits(&self, full_name: &str, number: u64) -> Result<Vec<CommitInfo>>;
    fn fetch_pr_files(&self, full_name: &str, number: u64) -> Result<Vec<DiffFile>>;
    fn fetch_pr_conversation(&self, full_name: &str, number: u64) -> Result<Vec<IssueComment>>;
    fn merge_pull_request(
        &self,
        full_name: &str,
        number: u64,
        commit_title: Option<&str>,
        merge_method: Option<&str>,
    ) -> Result<MergeResponse>;
    fn create_pull_request_review(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
        event: &str,
    ) -> Result<()>;
    fn update_pull_request(&self, full_name: &str, number: u64, state: &str) -> Result<()>;
    fn create_pull_request_comment(&self, full_name: &str, number: u64, body: &str) -> Result<()>;
    fn create_pull_request(
        &self,
        full_name: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
    ) -> Result<PullRequestDetail>;

    fn fetch_recent_commits(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<CommitInfo>>;
    fn fetch_compare(&self, full_name: &str, base: &str, head: &str) -> Result<CompareResponse>;

    fn fetch_check_runs(&self, full_name: &str, ref_: &str) -> Result<Vec<CheckRun>>;
    fn fetch_workflow_runs(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<WorkflowRun>>;

    fn fetch_releases(&self, full_name: &str, per_page: u8) -> Result<Vec<Release>>;
    fn create_release(
        &self,
        full_name: &str,
        tag_name: &str,
        name: Option<&str>,
        body: Option<&str>,
        prerelease: bool,
    ) -> Result<Release>;

    fn create_repo(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
    ) -> Result<RepoSummary>;

    fn fetch_languages(&self, full_name: &str) -> Result<Vec<LanguageStat>>;
    fn fetch_contributors(&self, full_name: &str, per_page: u8) -> Result<Vec<Contributor>>;

    fn rate_limit_remaining(&self) -> Option<u32>;
    fn rate_limit_reset(&self) -> Option<u64>;
}

/// Factory: create a provider instance for the given kind and token.
pub fn create_provider(kind: ProviderKind, token: Option<&str>) -> Result<Arc<dyn GitProvider>> {
    match kind {
        ProviderKind::GitHub | ProviderKind::Other => {
            let client = crate::github::GitHubClient::new(token)?;
            Ok(Arc::new(client))
        }
        _ => {
            let client = crate::github::GitHubClient::new(token)?;
            Ok(Arc::new(client))
        }
    }
}
