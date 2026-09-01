use super::{App, NetworkEvent};

impl App {
    pub(crate) fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::SearchResult(Ok(items)) => {
                if items.is_empty() && self.search.page > 1 {
                    self.search.page = self.search.page.saturating_sub(1);
                    self.status = "No more search results pages.".to_string();
                    return;
                }
                self.search.apply_results(items);
                self.clear_repo_context();
                self.status = format!(
                    "Loaded {} repositories on page {} (per_page {}).",
                    self.search.repos.len(),
                    self.search.page,
                    self.search.per_page
                );
            }
            NetworkEvent::SearchResult(Err(e)) => {
                if e.contains("401") {
                    self.status =
                        "Authentication required. Press t to set a GitHub token.".to_string();
                } else {
                    self.status = format!("Search failed: {e}");
                }
            }
            NetworkEvent::IssuesResult(Ok(issues)) => {
                let items = issues
                    .into_iter()
                    .map(|i| {
                        let status = if i.pull_request.is_some() {
                            "[PR]"
                        } else {
                            "[ISSUE]"
                        };
                        format!("{} #{}: {} ({})", status, i.number, i.title, i.state)
                    })
                    .collect();
                self.command_palette.show(items, false);
                self.status = "Issues loaded. Select with arrows, Enter to view.".to_string();
            }
            NetworkEvent::IssuesResult(Err(e)) => {
                self.status = format!("Issues fetch failed: {e}");
            }
            NetworkEvent::PrsResult(Ok(prs)) => {
                let items = prs
                    .into_iter()
                    .map(|pr| {
                        format!(
                            "[PR] #{}: {} ({} +{} -{})",
                            pr.number,
                            pr.title,
                            pr.state,
                            pr.additions.unwrap_or(0),
                            pr.deletions.unwrap_or(0)
                        )
                    })
                    .collect();
                self.command_palette.show(items, false);
                self.status = "Pull requests loaded.".to_string();
            }
            NetworkEvent::PrsResult(Err(e)) => {
                self.status = format!("PR fetch failed: {e}");
            }
            NetworkEvent::CommitsResult(Ok(commits)) => {
                let items = commits
                    .into_iter()
                    .map(|c| {
                        let short = c.sha.chars().take(7).collect::<String>();
                        let msg = c.commit.message.lines().next().unwrap_or("").to_string();
                        format!("[COMMIT] {} {} - {}", short, c.commit.author.name, msg)
                    })
                    .collect();
                self.command_palette.show(items, false);
                self.status = "Recent commits loaded.".to_string();
            }
            NetworkEvent::CommitsResult(Err(e)) => {
                self.status = format!("Commits fetch failed: {e}");
            }
            NetworkEvent::CompareResult(Ok(compare)) => {
                let items = compare
                    .files
                    .into_iter()
                    .map(|f| {
                        format!(
                            "[DIFF] {} ({} +{} -{})",
                            f.filename, f.status, f.additions, f.deletions
                        )
                    })
                    .collect();
                self.command_palette.show(items, false);
                self.status = format!(
                    "Compare: {} ahead, {} behind",
                    compare.ahead_by, compare.behind_by
                );
            }
            NetworkEvent::CompareResult(Err(e)) => {
                self.status = format!("Compare failed: {e}");
            }
            NetworkEvent::CheckRunsResult(Ok(runs)) => {
                let count = runs.len();
                let items = runs
                    .into_iter()
                    .map(|r| {
                        let conclusion = r.conclusion.as_deref().unwrap_or("pending");
                        format!("[CI] {}: {}", r.name, conclusion)
                    })
                    .collect();
                self.command_palette.show(items, false);
                self.status = format!("CI checks: {}", count);
            }
            NetworkEvent::CheckRunsResult(Err(e)) => {
                self.status = format!("CI check fetch failed: {e}");
            }
            NetworkEvent::StarredResult(Ok(repos)) => {
                self.search.apply_results(repos);
                self.clear_repo_context();
                self.status = format!("Loaded {} starred repositories.", self.search.repos.len());
            }
            NetworkEvent::StarredResult(Err(e)) => {
                self.status = format!("Starred repos fetch failed: {e}");
            }
            NetworkEvent::PrDetailResult(Ok(detail)) => {
                self.pr_detail = Some(detail.clone());
                let mut items = vec![
                    format!("#{}: {}", detail.number, detail.title),
                    format!(
                        "State: {} | Files: {} | +/-: {}/{}",
                        detail.state,
                        detail.changed_files.unwrap_or(0),
                        detail.additions.unwrap_or(0),
                        detail.deletions.unwrap_or(0)
                    ),
                    format!(
                        "{} {} -> {} {}",
                        detail.base.label,
                        detail.base.sha.chars().take(7).collect::<String>(),
                        detail.head.label,
                        detail.head.sha.chars().take(7).collect::<String>()
                    ),
                ];
                if let Some(body) = &detail.body {
                    for line in body.lines().take(5) {
                        items.push(format!("  {line}"));
                    }
                }
                if detail.state == "open" {
                    items.push("[Approve]".to_string());
                    items.push("[Request Changes]".to_string());
                    items.push("[Comment]".to_string());
                    items.push("[Merge: merge commit]".to_string());
                    items.push("[Merge: squash]".to_string());
                    items.push("[Merge: rebase]".to_string());
                    items.push("[Close PR]".to_string());
                }
                items.push("[View Reviews]".to_string());
                items.push("[View Comments]".to_string());
                items.push("[View Commits]".to_string());
                self.command_palette.show(items, true);
                self.status = format!("PR #{}: {}", detail.number, detail.title);
            }
            NetworkEvent::PrDetailResult(Err(e)) => {
                self.status = format!("PR detail fetch failed: {e}");
            }
            NetworkEvent::PrReviewsResult(Ok(reviews)) => {
                self.pr_reviews = reviews;
                let mut items = self
                    .pr_reviews
                    .iter()
                    .map(|r| {
                        let short_body = r
                            .body
                            .as_deref()
                            .unwrap_or("")
                            .chars()
                            .take(80)
                            .collect::<String>();
                        format!("[REVIEW] {}: {} - {}", r.user.login, r.state, short_body)
                    })
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    items.push("No reviews yet.".to_string());
                }
                self.command_palette.show(items, true);
                self.status = format!("{} reviews loaded.", self.pr_reviews.len());
            }
            NetworkEvent::PrReviewsResult(Err(e)) => {
                self.status = format!("Reviews fetch failed: {e}");
            }
            NetworkEvent::PrCommentsResult(Ok(comments)) => {
                self.pr_comments = comments;
                let mut items = self
                    .pr_comments
                    .iter()
                    .map(|c| {
                        let path_info = c
                            .path
                            .as_deref()
                            .map(|p| format!(" on {}", p))
                            .unwrap_or_default();
                        format!("[COMMENT]{}{}", c.user.login, path_info)
                    })
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    items.push("No comments yet.".to_string());
                }
                self.command_palette.show(items, true);
                self.status = format!("{} comments loaded.", self.pr_comments.len());
            }
            NetworkEvent::PrCommentsResult(Err(e)) => {
                self.status = format!("Comments fetch failed: {e}");
            }
            NetworkEvent::PrCommitsResult(Ok(commits)) => {
                let mut items = commits
                    .iter()
                    .map(|c| {
                        let short = c.sha.chars().take(7).collect::<String>();
                        let msg = c
                            .commit
                            .message
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(80)
                            .collect::<String>();
                        format!("{} {} - {}", short, c.commit.author.name, msg)
                    })
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    items.push("No commits.".to_string());
                }
                self.command_palette.show(items, true);
                self.status = format!("{} commits loaded.", commits.len());
            }
            NetworkEvent::PrCommitsResult(Err(e)) => {
                self.status = format!("Commits fetch failed: {e}");
            }
            NetworkEvent::PrMergeResult(Ok(resp)) => {
                if resp.merged {
                    self.status = format!(
                        "PR merged! sha: {}",
                        resp.sha.chars().take(7).collect::<String>()
                    );
                } else {
                    self.status = format!("Merge failed: {}", resp.message);
                }
                self.pr_detail = None;
            }
            NetworkEvent::PrMergeResult(Err(e)) => {
                self.status = format!("Merge failed: {e}");
            }
            NetworkEvent::PrActionResult(msg) => {
                self.status = msg;
            }
            NetworkEvent::WorkflowRunsResult(runs) => {
                let count = runs.len();
                let items = runs
                    .into_iter()
                    .map(|r| {
                        let conclusion = r.conclusion.as_deref().unwrap_or("pending");
                        format!("[WF] {}: {}", r.name, conclusion)
                    })
                    .collect();
                self.command_palette.show(items, false);
                self.status = format!("Workflow runs: {}", count);
            }
        }
    }
}
