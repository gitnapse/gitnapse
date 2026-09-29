use crate::github::GitHubClient;
use crate::provider::GitProvider;
use anyhow::Result;

impl GitProvider for GitHubClient {
    fn fetch_authenticated_user(&self) -> Result<Option<String>> {
        self.fetch_authenticated_user().map_err(Into::into)
    }

    fn search_repositories_page(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::RepoSummary>> {
        self.search_repositories_page(query, page, per_page)
            .map_err(Into::into)
    }

    fn fetch_branches(&self, full_name: &str) -> Result<Vec<String>> {
        self.fetch_branches(full_name).map_err(Into::into)
    }

    fn fetch_repo_tree(
        &self,
        full_name: &str,
        branch: &str,
    ) -> Result<Vec<crate::models::RepoNode>> {
        self.fetch_repo_tree(full_name, branch).map_err(Into::into)
    }

    fn fetch_starred_repos(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::RepoSummary>> {
        self.fetch_starred_repos(page, per_page).map_err(Into::into)
    }

    fn fetch_repo_by_name(&self, full_name: &str) -> Result<crate::models::RepoSummary> {
        self.fetch_repo_by_name(full_name).map_err(Into::into)
    }

    fn fetch_user_profile(&self, login: &str) -> Result<crate::models::UserProfile> {
        self.fetch_user_profile(login).map_err(Into::into)
    }

    fn fetch_user_repos(
        &self,
        login: &str,
        sort: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::RepoSummary>> {
        self.fetch_user_repos(login, sort, page, per_page)
            .map_err(Into::into)
    }

    fn search_users(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::UserProfile>> {
        self.search_users(query, page, per_page).map_err(Into::into)
    }

    fn fetch_user_events(
        &self,
        login: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::UserEvent>> {
        self.fetch_user_events(login, page, per_page)
            .map_err(Into::into)
    }

    fn fetch_notifications(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::Notification>> {
        self.fetch_notifications(page, per_page).map_err(Into::into)
    }

    fn mark_notification_read(&self, id: &str) -> Result<()> {
        self.mark_notification_read(id).map_err(Into::into)
    }

    fn search_code(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<crate::models::CodeSearchResult>> {
        self.search_code(query, page, per_page).map_err(Into::into)
    }

    fn fetch_file_content(&self, full_name: &str, path: &str) -> Result<Vec<u8>> {
        self.fetch_file_content(full_name, path)
    }

    fn fetch_file_content_by_ref(
        &self,
        full_name: &str,
        path: &str,
        git_ref: &str,
    ) -> Result<Vec<u8>> {
        self.fetch_file_content_by_ref(full_name, path, git_ref)
    }

    fn fetch_issues(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<crate::models::Issue>> {
        self.fetch_issues(full_name, state, per_page)
            .map_err(Into::into)
    }

    fn fetch_issue_detail(&self, full_name: &str, number: u64) -> Result<crate::models::Issue> {
        self.fetch_issue_detail(full_name, number)
            .map_err(Into::into)
    }

    fn fetch_issue_comments(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<crate::models::IssueComment>> {
        self.fetch_issue_comments(full_name, number)
            .map_err(Into::into)
    }

    fn create_issue_comment(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
    ) -> Result<crate::models::IssueComment> {
        self.create_issue_comment(full_name, number, body)
            .map_err(Into::into)
    }

    fn reopen_issue(&self, full_name: &str, number: u64) -> Result<crate::models::Issue> {
        self.reopen_issue(full_name, number).map_err(Into::into)
    }

    fn create_issue(
        &self,
        full_name: &str,
        title: &str,
        body: Option<&str>,
    ) -> Result<crate::models::Issue> {
        self.create_issue(full_name, title, body)
            .map_err(Into::into)
    }

    fn close_issue(&self, full_name: &str, number: u64) -> Result<crate::models::Issue> {
        self.close_issue(full_name, number).map_err(Into::into)
    }

    fn fetch_pull_requests(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<crate::models::PullRequest>> {
        self.fetch_pull_requests(full_name, state, per_page)
            .map_err(Into::into)
    }

    fn fetch_pull_request_detail(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<crate::models::PullRequestDetail> {
        self.fetch_pull_request_detail(full_name, number)
            .map_err(Into::into)
    }

    fn fetch_pull_request_reviews(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<crate::models::PullRequestReview>> {
        self.fetch_pull_request_reviews(full_name, number)
            .map_err(Into::into)
    }

    fn fetch_pull_request_comments(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<crate::models::ReviewComment>> {
        self.fetch_pull_request_comments(full_name, number)
            .map_err(Into::into)
    }

    fn fetch_pull_request_commits(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<crate::models::CommitInfo>> {
        self.fetch_pull_request_commits(full_name, number)
            .map_err(Into::into)
    }

    fn fetch_pr_files(&self, full_name: &str, number: u64) -> Result<Vec<crate::models::DiffFile>> {
        self.fetch_pr_files(full_name, number).map_err(Into::into)
    }

    fn fetch_pr_conversation(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<crate::models::IssueComment>> {
        self.fetch_pr_conversation(full_name, number)
            .map_err(Into::into)
    }

    fn merge_pull_request(
        &self,
        full_name: &str,
        number: u64,
        commit_title: Option<&str>,
        merge_method: Option<&str>,
    ) -> Result<crate::models::MergeResponse> {
        self.merge_pull_request(full_name, number, commit_title, merge_method)
            .map_err(Into::into)
    }

    fn create_pull_request_review(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
        event: &str,
    ) -> Result<()> {
        let _ = self.create_pull_request_review(full_name, number, body, event)?;
        Ok(())
    }

    fn update_pull_request(&self, full_name: &str, number: u64, state: &str) -> Result<()> {
        let _ = self.update_pull_request(full_name, number, state)?;
        Ok(())
    }

    fn create_pull_request_comment(&self, full_name: &str, number: u64, body: &str) -> Result<()> {
        let _ = self.create_pull_request_comment(full_name, number, body)?;
        Ok(())
    }

    fn create_pull_request(
        &self,
        full_name: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
    ) -> Result<crate::models::PullRequestDetail> {
        self.create_pull_request(full_name, title, head, base, body)
            .map_err(Into::into)
    }

    fn fetch_recent_commits(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<crate::models::CommitInfo>> {
        self.fetch_recent_commits(full_name, branch, per_page)
            .map_err(Into::into)
    }

    fn fetch_compare(
        &self,
        full_name: &str,
        base: &str,
        head: &str,
    ) -> Result<crate::models::CompareResponse> {
        self.fetch_compare(full_name, base, head)
            .map_err(Into::into)
    }

    fn fetch_check_runs(
        &self,
        full_name: &str,
        ref_: &str,
    ) -> Result<Vec<crate::models::CheckRun>> {
        self.fetch_check_runs(full_name, ref_).map_err(Into::into)
    }

    fn fetch_workflow_runs(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<crate::models::WorkflowRun>> {
        self.fetch_workflow_runs(full_name, branch, per_page)
            .map_err(Into::into)
    }

    fn fetch_releases(&self, full_name: &str, per_page: u8) -> Result<Vec<crate::models::Release>> {
        self.fetch_releases(full_name, per_page).map_err(Into::into)
    }

    fn create_release(
        &self,
        full_name: &str,
        tag_name: &str,
        name: Option<&str>,
        body: Option<&str>,
        prerelease: bool,
    ) -> Result<crate::models::Release> {
        self.create_release(full_name, tag_name, name, body, prerelease)
            .map_err(Into::into)
    }

    fn create_repo(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
    ) -> Result<crate::models::RepoSummary> {
        self.create_repo(name, description, private)
            .map_err(Into::into)
    }

    fn fetch_languages(&self, full_name: &str) -> Result<Vec<crate::models::LanguageStat>> {
        self.fetch_languages(full_name).map_err(Into::into)
    }

    fn fetch_contributors(
        &self,
        full_name: &str,
        per_page: u8,
    ) -> Result<Vec<crate::models::Contributor>> {
        self.fetch_contributors(full_name, per_page)
            .map_err(Into::into)
    }

    fn rate_limit_remaining(&self) -> Option<u32> {
        self.rate_limit_remaining()
    }

    fn rate_limit_reset(&self) -> Option<u64> {
        self.rate_limit_reset()
    }
}
