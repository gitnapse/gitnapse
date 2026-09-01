use crate::app::{App, Focus, NetworkEvent, screens::Screen};
use crate::provider::GitProvider;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Line;
use std::sync::Arc;
use std::sync::mpsc;

impl App {
    pub(super) fn repo_window(&self, area_height: u16) -> (usize, usize) {
        self.search.window(area_height)
    }

    fn back_to_repo_list(&mut self) {
        if self.current_repo.is_some() {
            self.current_repo = None;
            self.tree_all.clear();
            self.tree_visible_limit = 0;
            self.selected_node = 0;
            self.branches.clear();
            self.selected_branch = 0;
            self.preview_title = "Preview".to_string();
            self.preview_lines = vec![Line::from("Select a repository and a file to preview.")];
            self.preview_scroll = 0;
            self.current_preview_path = None;
            self.tree_text_mode = false;
            self.focus = Focus::Repos;
            self.status = "Returned to repository search list.".to_string();
        } else {
            self.focus = Focus::Repos;
        }
    }

    fn handle_navigation(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("quit", &code) {
            self.should_quit = true;
        } else if code == KeyCode::Char(' ')
            && self.focus == Focus::Repos
            && !self.search.repos.is_empty()
        {
            let count = self.search.toggle_multi_select();
            self.status = format!("{} repository(s) selected", count);
        } else if self.keybindings.matches_key("search", &code) {
            self.focus = Focus::Search;
            self.input_buffer = self.search.query.clone();
        } else if self.keybindings.matches_key("token_input", &code) {
            self.focus = Focus::TokenInput;
            self.input_buffer.clear();
        } else if self.keybindings.matches_key("oauth_status", &code) {
            self.run_oauth_quick_check();
        } else if self.keybindings.matches_key("clone", &code) {
            if self.current_repo.is_some() {
                self.clone_path_input = self.account.preferred_clone_dir.clone();
                self.focus = Focus::ClonePath;
            } else {
                self.status = "Open a repository first, then press c to clone.".to_string();
            }
        } else if self.keybindings.matches_key("branch_picker", &code) {
            if self.current_repo.is_some() && !self.branches.is_empty() {
                self.focus = Focus::BranchPicker;
            } else {
                self.status = "Open a repository first to select a branch.".to_string();
            }
        } else if self.keybindings.matches_key("file_search", &code) {
            if self.current_repo.is_some() {
                self.tree_search_input.clear();
                self.focus = Focus::TreeSearch;
            }
        } else if self.keybindings.matches_key("download", &code) {
            if self.current_preview_path.is_some() {
                self.download_path_input = ".".to_string();
                self.focus = Focus::DownloadPath;
            } else {
                self.status = "Preview a file first before downloading.".to_string();
            }
        } else if self.keybindings.matches_key("tree_view", &code) {
            self.toggle_tree_view();
        } else if self.keybindings.matches_key("focus_next", &code) {
            self.focus = match self.focus {
                Focus::Repos if !self.tree_all.is_empty() => Focus::Tree,
                Focus::Tree if !self.preview_lines.is_empty() => Focus::Preview,
                _ => Focus::Repos,
            }
        } else if self.keybindings.matches_key("escape", &code) {
            self.back_to_repo_list();
        } else if self.keybindings.matches_key("page_left", &code) {
            if self.focus == Focus::Repos && self.search.page > 1 {
                self.search.page = self.search.page.saturating_sub(1);
                self.search();
            }
        } else if self.keybindings.matches_key("page_right", &code) {
            if self.focus == Focus::Repos {
                self.search.page = self.search.page.saturating_add(1);
                self.search();
            }
        } else if self.keybindings.matches_key("scroll_down", &code) {
            if self.focus == Focus::Tree && !self.tree_all.is_empty() {
                self.selected_node =
                    (self.selected_node + 1).min(self.tree_all.len().saturating_sub(1));
                self.ensure_lazy_tree_progress();
            } else if self.focus == Focus::Preview {
                self.scroll_preview_down(1, self.preview_viewport_rows);
            } else if !self.search.repos.is_empty() {
                self.search.move_selection(1);
            }
        } else if self.keybindings.matches_key("scroll_up", &code) {
            if self.focus == Focus::Tree && !self.tree_all.is_empty() {
                self.selected_node = self.selected_node.saturating_sub(1);
            } else if self.focus == Focus::Preview {
                self.scroll_preview_up(1);
            } else if !self.search.repos.is_empty() {
                self.search.move_selection(-1);
            }
        } else if self.keybindings.matches_key("page_down", &code) {
            if self.focus == Focus::Preview {
                self.scroll_preview_down(
                    self.preview_viewport_rows / 2,
                    self.preview_viewport_rows,
                );
            }
        } else if self.keybindings.matches_key("page_up", &code) {
            if self.focus == Focus::Preview {
                self.scroll_preview_up(self.preview_viewport_rows / 2);
            }
        } else if self.keybindings.matches_key("home", &code) {
            if self.focus == Focus::Preview {
                self.preview_scroll = 0;
            }
        } else if self.keybindings.matches_key("end", &code) {
            if self.focus == Focus::Preview {
                self.preview_scroll = self.preview_lines.len().saturating_sub(1);
            }
        } else if self.keybindings.matches_key("enter", &code) {
            if self.focus == Focus::Tree {
                self.preview_selected_file();
                self.focus = Focus::Preview;
            } else {
                self.open_selected_repo();
            }
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match self.focus {
            Focus::Search => self.handle_search_input(code),
            Focus::TreeSearch => self.handle_tree_search_input(code),
            Focus::DownloadPath => self.handle_download_path_input(code),
            Focus::ClonePath => self.handle_clone_path_input(code),
            Focus::TokenInput => self.handle_token_input(code),
            Focus::OAuthClientIdInput => self.handle_oauth_client_id_input(code),
            Focus::BranchPicker => self.handle_branch_picker_input(code),
            Focus::Repos | Focus::Tree | Focus::Preview => self.handle_navigation(code),
        }
    }

    pub(crate) fn handle_key_with_channel(
        &mut self,
        event: KeyEvent,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<dyn GitProvider>,
    ) {
        let code = event.code;

        // Info overlay – any key closes it.
        if self.show_info {
            self.show_info = false;
            return;
        }
        // PR review / creation input mode
        if self.pr_pending_action.is_some() {
            if self.keybindings.matches_key("escape", &code) {
                self.pr_pending_action = None;
                self.pr_pending_body.clear();
                self.status = "Action cancelled.".to_string();
            } else if self.keybindings.matches_key("enter", &code) {
                let text = self.pr_pending_body.trim().to_string();
                let action = self.pr_pending_action.take().unwrap_or_default();
                let Some(repo) = self.current_repo.clone() else {
                    self.status = "No repository loaded.".to_string();
                    return;
                };
                let Some(detail) = self.pr_detail.clone() else {
                    // For create_pr, detail is not needed
                    if action.starts_with("create_pr_") {
                        self.handle_pr_creation_step(action, text, tx, github);
                    } else {
                        self.status = "No PR loaded.".to_string();
                    }
                    return;
                };
                let full_name = repo.full_name.clone();
                let number = detail.number;
                let g = github.clone();

                match action.as_str() {
                    "approve" => {
                        self.status = "Approving PR...".to_string();
                        let tx = tx.clone();
                        self.task_manager.spawn(move || {
                            let body = if text.is_empty() {
                                "LGTM, approved."
                            } else {
                                &text
                            };
                            match g.create_pull_request_review(&full_name, number, body, "APPROVE")
                            {
                                Ok(_) => {
                                    let _ = tx.send(NetworkEvent::PrActionResult(
                                        "PR approved.".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = tx.send(NetworkEvent::PrActionResult(format!(
                                        "Approve failed: {e}"
                                    )));
                                }
                            }
                        });
                    }
                    "request_changes" => {
                        self.status = "Requesting changes...".to_string();
                        let tx = tx.clone();
                        self.task_manager.spawn(move || {
                            let body = if text.is_empty() {
                                "Please address the requested changes."
                            } else {
                                &text
                            };
                            match g.create_pull_request_review(
                                &full_name,
                                number,
                                body,
                                "REQUEST_CHANGES",
                            ) {
                                Ok(_) => {
                                    let _ = tx.send(NetworkEvent::PrActionResult(
                                        "Changes requested.".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = tx.send(NetworkEvent::PrActionResult(format!(
                                        "Request failed: {e}"
                                    )));
                                }
                            }
                        });
                    }
                    "comment" => {
                        self.status = "Posting comment...".to_string();
                        let tx = tx.clone();
                        self.task_manager.spawn(move || {
                            let body = if text.is_empty() {
                                "Reviewed the changes."
                            } else {
                                &text
                            };
                            match g.create_pull_request_review(&full_name, number, body, "COMMENT")
                            {
                                Ok(_) => {
                                    let _ = tx.send(NetworkEvent::PrActionResult(
                                        "Comment posted.".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = tx.send(NetworkEvent::PrActionResult(format!(
                                        "Comment failed: {e}"
                                    )));
                                }
                            }
                        });
                    }
                    _ => {
                        self.status = format!("Unknown action: {action}");
                    }
                }
                self.pr_pending_body.clear();
            } else if self.keybindings.matches_key("backspace", &code) {
                self.pr_pending_body.pop();
            } else if let KeyCode::Char(ch) = code {
                self.pr_pending_body.push(ch);
            }
            return;
        }

        // PR number input mode
        if self.focus == Focus::TreeSearch
            && self.pr_detail.is_none()
            && !self.command_palette.visible
        {
            if self.keybindings.matches_key("escape", &code) {
                self.tree_search_input.clear();
                self.focus = Focus::Tree;
                self.status = "PR number input cancelled.".to_string();
            } else if self.keybindings.matches_key("enter", &code) {
                let input = self.tree_search_input.trim().to_string();
                if let Ok(number) = input.parse::<u64>() {
                    if let Some(repo) = self.current_repo.clone() {
                        self.status = format!("Loading PR #{number}...");
                        self.focus = Focus::Tree;
                        let g = github.clone();
                        let full_name = repo.full_name.clone();
                        let tx = tx.clone();
                        self.task_manager.spawn(move || {
                            let result = g.fetch_pull_request_detail(&full_name, number);
                            let _ = tx.send(NetworkEvent::PrDetailResult(
                                result.map_err(|e| e.to_string()),
                            ));
                        });
                    }
                } else {
                    self.status = format!("Invalid PR number: {input}");
                }
            } else if self.keybindings.matches_key("backspace", &code) {
                self.tree_search_input.pop();
            } else if let KeyCode::Char(ch) = code {
                self.tree_search_input.push(ch);
            }
            return;
        }

        if self.command_palette.visible {
            let outcome = self.command_palette.handle_key(&self.keybindings, event);
            self.perform_palette_outcome(outcome, tx, github);
            return;
        }

        if self
            .keybindings
            .matches_key_with_mods("command_palette", &code, &event.modifiers)
        {
            self.toggle_command_palette();
            return;
        }

        self.handle_key(code);
    }
}
