use super::{App, Focus, NetworkEvent, theme};
use crate::app::screens::command_palette::{CommandPalette, PaletteOutcome};
use crate::provider::GitProvider;
use secrecy::SecretString;
use std::sync::Arc;
use std::sync::mpsc;

impl App {
    pub(crate) fn toggle_command_palette(&mut self) {
        let opening = !self.command_palette.visible;
        self.command_palette.visible = opening;
        if opening {
            self.command_palette.input.clear();
            self.command_palette.cursor = 0;
            self.command_palette.is_pr_action = false;
            self.command_palette.is_theme_picker = false;
            let items = CommandPalette::command_list(self);
            self.command_palette.items = items;
        }
    }

    /// Perform an outcome produced by the command palette.
    pub(crate) fn perform_palette_outcome(
        &mut self,
        outcome: PaletteOutcome,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<dyn GitProvider>,
    ) {
        match outcome {
            PaletteOutcome::None => {}
            PaletteOutcome::ExecuteCommand(cmd) => self.execute_command(cmd, tx, github),
            PaletteOutcome::ExecutePrAction(action) => self.execute_pr_action(action, tx, github),
            PaletteOutcome::ThemePicked(name) => self.apply_theme(name),
        }
    }

    fn apply_theme(&mut self, theme_name: String) {
        let config = theme::load_theme_by_name(&theme_name);
        theme::init_theme(&config);
        self.status = format!("Theme changed to {theme_name}.");
    }

    fn execute_command(
        &mut self,
        cmd: String,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<dyn GitProvider>,
    ) {
        match cmd.as_str() {
            "Search Repositories" => {
                self.focus = Focus::Search;
                self.input_buffer = self.search.query.clone();
            }
            "List Starred Repos" => {
                self.status = "Loading starred repos...".to_string();
                let g = github.clone();
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    let result = g.fetch_starred_repos(1, 30);
                    let _ = tx.send(NetworkEvent::StarredResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });
            }
            "Switch Branch" => {
                if self.current_repo.is_some() && !self.branches.is_empty() {
                    self.focus = Focus::BranchPicker;
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Find File" => {
                if self.current_repo.is_some() {
                    self.tree_search_input.clear();
                    self.focus = Focus::TreeSearch;
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Clone Repository" => {
                if self.current_repo.is_some() {
                    self.clone_path_input = self.account.preferred_clone_dir.clone();
                    self.focus = Focus::ClonePath;
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Download Current File" => {
                if self.current_preview_path.is_some() {
                    self.download_path_input = ".".to_string();
                    self.focus = Focus::DownloadPath;
                } else {
                    self.status = "Preview a file first.".to_string();
                }
            }
            "Toggle Tree View" => {
                self.toggle_tree_view();
            }
            "Show Info" => {
                self.show_info = true;
                self.status = "GitNapse info. Press Esc to close.".to_string();
            }
            "Change Theme" => {
                let themes = theme::list_available_themes();
                if themes.is_empty() {
                    self.status = "No themes found.".to_string();
                } else {
                    self.command_palette.enter_theme_picker(themes);
                }
            }
            "Set Token" => {
                self.focus = Focus::TokenInput;
                self.token_buffer = SecretString::new(String::new().into());
                self.input_buffer.clear();
            }
            "List Issues" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading issues...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let tx = tx.clone();
                    self.task_manager.spawn(move || {
                        let result = g.fetch_issues(&full_name, "open", 30);
                        let _ = tx.send(NetworkEvent::IssuesResult(
                            result.map_err(|e| e.to_string()),
                        ));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "List Pull Requests" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading pull requests...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let tx = tx.clone();
                    self.task_manager.spawn(move || {
                        let result = g.fetch_pull_requests(&full_name, "open", 30);
                        let _ = tx.send(NetworkEvent::PrsResult(result.map_err(|e| e.to_string())));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "View Recent Commits" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading commits...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let branch = self.selected_branch_name();
                    let tx = tx.clone();
                    self.task_manager.spawn(move || {
                        let result = g.fetch_recent_commits(&full_name, &branch, 30);
                        let _ = tx.send(NetworkEvent::CommitsResult(
                            result.map_err(|e| e.to_string()),
                        ));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "View CI Status" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading CI status...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let branch = self.selected_branch_name();
                    let tx = tx.clone();
                    self.task_manager.spawn(move || {
                        let result = g.fetch_check_runs(&full_name, &branch);
                        let _ = tx.send(NetworkEvent::CheckRunsResult(
                            result.map_err(|e| e.to_string()),
                        ));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "View Workflow Runs" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading workflow runs...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let branch = self.selected_branch_name();
                    let tx = tx.clone();
                    self.task_manager.spawn(move || {
                        let result = g.fetch_workflow_runs(&full_name, &branch, 30);
                        let _ =
                            tx.send(NetworkEvent::WorkflowRunsResult(result.unwrap_or_default()));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "View PR Detail" => {
                self.pending_pr_number.clear();
                self.tree_search_input.clear();
                self.focus = Focus::TreeSearch; // reuse input for PR number
                self.status = "Enter PR number and press Enter:".to_string();
            }
            "Create Pull Request" => {
                self.pr_pending_action = Some("create_pr_title".to_string());
                self.pr_pending_body.clear();
                self.status = "Enter PR title:".to_string();
            }
            "Compare Branches" => {
                if let Some(repo) = self.current_repo.clone() {
                    if self.branches.len() >= 2 {
                        let base = self.selected_branch_name();
                        let head = self
                            .branches
                            .iter()
                            .find(|b| **b != base)
                            .cloned()
                            .unwrap_or_default();
                        if !head.is_empty() {
                            self.status = format!("Comparing {base}...{head}");
                            let g = github.clone();
                            let full_name = repo.full_name.clone();
                            let base = base.clone();
                            let head = head.clone();
                            let tx = tx.clone();
                            self.task_manager.spawn(move || {
                                let result = g.fetch_compare(&full_name, &base, &head);
                                let _ = tx.send(NetworkEvent::CompareResult(
                                    result.map_err(|e| e.to_string()),
                                ));
                            });
                        }
                    } else {
                        self.status = "Need at least 2 branches loaded.".to_string();
                    }
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Quit" => {
                self.should_quit = true;
            }
            cmd if cmd.starts_with("Clone ") && cmd.contains("Selected Repos") => {
                if self.search.multi_selected.is_empty() {
                    self.status = "No repos selected.".to_string();
                } else {
                    let selected: Vec<_> = self
                        .search
                        .multi_selected
                        .iter()
                        .filter_map(|&i| self.search.repos.get(i))
                        .cloned()
                        .collect();
                    let dest = self.clone_path_input.trim().to_string();
                    if dest.is_empty() {
                        self.status = "Clone destination not set.".to_string();
                    } else {
                        let count = selected.len();
                        self.status = format!("Cloning {} repo(s) to {}...", count, dest);
                        self.task_manager.spawn(move || {
                            for repo in &selected {
                                let repo_path = std::path::PathBuf::from(&dest).join(&repo.name);
                                if repo_path.exists() {
                                    log::info!("Skipping {} (already exists)", repo.full_name);
                                    continue;
                                }
                                let _ = std::process::Command::new("git")
                                    .arg("clone")
                                    .arg(&repo.clone_url)
                                    .arg(&repo_path)
                                    .output();
                            }
                        });
                        self.search.multi_selected.clear();
                    }
                }
            }
            _ => {
                self.status = format!("Unknown command: {cmd}");
            }
        }
    }

    fn execute_pr_action(
        &mut self,
        action: String,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<dyn GitProvider>,
    ) {
        match action.as_str() {
            "[Approve]" => {
                self.pr_pending_action = Some("approve".to_string());
                self.pr_pending_body.clear();
                self.status = "Enter approval comment (or empty for default):".to_string();
            }
            "[Request Changes]" => {
                self.pr_pending_action = Some("request_changes".to_string());
                self.pr_pending_body.clear();
                self.status = "Enter change request description:".to_string();
            }
            "[Comment]" => {
                self.pr_pending_action = Some("comment".to_string());
                self.pr_pending_body.clear();
                self.status = "Enter your review comment:".to_string();
            }
            method if method.starts_with("[Merge:") => {
                let merge_method = method
                    .strip_prefix("[Merge: ")
                    .and_then(|s| s.strip_suffix(']'))
                    .unwrap_or("merge");
                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repository loaded.".to_string();
                    return;
                };
                let Some(detail) = self.pr_detail.clone() else {
                    self.status = "No PR loaded.".to_string();
                    return;
                };
                self.status = format!("Merging PR ({merge_method})...");
                let g = github.clone();
                let full_name = repo.full_name.clone();
                let number = detail.number;
                let method = merge_method.to_string();
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    let result = g.merge_pull_request(&full_name, number, None, Some(&method));
                    let _ = tx.send(NetworkEvent::PrMergeResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });
            }
            "[Close PR]" => {
                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repository loaded.".to_string();
                    return;
                };
                let Some(detail) = self.pr_detail.clone() else {
                    self.status = "No PR loaded.".to_string();
                    return;
                };
                self.status = "Closing PR...".to_string();
                let g = github.clone();
                let full_name = repo.full_name.clone();
                let number = detail.number;
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    match g.update_pull_request(&full_name, number, "closed") {
                        Ok(_) => {
                            let _ = tx.send(NetworkEvent::PrActionResult("PR closed.".to_string()));
                        }
                        Err(e) => {
                            let _ =
                                tx.send(NetworkEvent::PrActionResult(format!("Close failed: {e}")));
                        }
                    }
                });
            }
            "[View Reviews]" => {
                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repo.".to_string();
                    return;
                };
                let Some(detail) = self.pr_detail.clone() else {
                    self.status = "No PR.".to_string();
                    return;
                };
                self.status = "Loading reviews...".to_string();
                let g = github.clone();
                let full_name = repo.full_name.clone();
                let number = detail.number;
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    let result = g.fetch_pull_request_reviews(&full_name, number);
                    let _ = tx.send(NetworkEvent::PrReviewsResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });
            }
            "[View Comments]" => {
                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repo.".to_string();
                    return;
                };
                let Some(detail) = self.pr_detail.clone() else {
                    self.status = "No PR.".to_string();
                    return;
                };
                self.status = "Loading comments...".to_string();
                let g = github.clone();
                let full_name = repo.full_name.clone();
                let number = detail.number;
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    let result = g.fetch_pull_request_comments(&full_name, number);
                    let _ = tx.send(NetworkEvent::PrCommentsResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });
            }
            "[View Commits]" => {
                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repo.".to_string();
                    return;
                };
                let Some(detail) = self.pr_detail.clone() else {
                    self.status = "No PR.".to_string();
                    return;
                };
                self.status = "Loading commits...".to_string();
                let g = github.clone();
                let full_name = repo.full_name.clone();
                let number = detail.number;
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    let result = g.fetch_pull_request_commits(&full_name, number);
                    let _ = tx.send(NetworkEvent::PrCommitsResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });
            }
            _ => {
                self.status = format!("Unknown action: {action}");
            }
        }
    }

    pub(crate) fn handle_pr_creation_step(
        &mut self,
        action: String,
        text: String,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<dyn GitProvider>,
    ) {
        match action.as_str() {
            "create_pr_title" => {
                if text.is_empty() {
                    self.status = "Title cannot be empty. Enter PR title:".to_string();
                    self.pr_pending_action = Some("create_pr_title".to_string());
                    return;
                }
                self.pr_pending_body = text;
                self.pr_pending_action = Some("create_pr_head".to_string());
                self.status = "Enter head branch (source):".to_string();
            }
            "create_pr_head" => {
                if text.is_empty() {
                    self.status = "Head branch cannot be empty. Enter head branch:".to_string();
                    self.pr_pending_action = Some("create_pr_head".to_string());
                    return;
                }
                self.pr_pending_body = format!("{}\n{}", self.pr_pending_body, text);
                self.pr_pending_action = Some("create_pr_base".to_string());
                self.status = "Enter base branch (target, e.g. main):".to_string();
            }
            "create_pr_base" => {
                if text.is_empty() {
                    self.status = "Base branch cannot be empty. Enter base branch:".to_string();
                    self.pr_pending_action = Some("create_pr_base".to_string());
                    return;
                }
                let parts: Vec<&str> = self.pr_pending_body.splitn(2, '\n').collect();
                let title = parts.first().unwrap_or(&"").to_string();
                let head = parts.get(1).unwrap_or(&"").to_string();
                let base = text;

                self.status = "Enter PR description (optional, or empty to skip):".to_string();
                // Store as: title\nhead\nbase
                self.pr_pending_body = format!("{title}\n{head}\n{base}");
                self.pr_pending_action = Some("create_pr_body".to_string());
            }
            "create_pr_body" => {
                let parts: Vec<&str> = self.pr_pending_body.splitn(3, '\n').collect();
                let title = parts.first().unwrap_or(&"").to_string();
                let head = parts.get(1).unwrap_or(&"").to_string();
                let base = parts.get(2).unwrap_or(&"").to_string();
                let body = if text.is_empty() { None } else { Some(text) };

                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repository loaded.".to_string();
                    self.pr_pending_action = None;
                    self.pr_pending_body.clear();
                    return;
                };

                self.status = format!("Creating PR: {title}");
                let g = github.clone();
                let full_name = repo.full_name.clone();
                let body_clone = body.clone();
                let tx = tx.clone();
                self.task_manager.spawn(move || {
                    let result = g.create_pull_request(
                        &full_name,
                        &title,
                        &head,
                        &base,
                        body_clone.as_deref(),
                    );
                    let _ = tx.send(NetworkEvent::PrDetailResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });

                self.pr_pending_action = None;
                self.pr_pending_body.clear();
            }
            _ => {
                self.status = format!("Unknown creation step: {action}");
                self.pr_pending_action = None;
                self.pr_pending_body.clear();
            }
        }
    }
}
