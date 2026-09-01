use super::{App, Focus, TerminalGuard};
use crate::auth;
use crate::oauth;
use crate::syntax::highlight_content;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use ratatui::text::Line;
use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;

impl App {
    pub(crate) fn search(&mut self) {
        self.status = "Loading...".to_string();
        match self.github.search_repositories_page(
            &self.search.query,
            self.search.page,
            self.search.per_page,
        ) {
            Ok(items) => {
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
            Err(error) => {
                self.status = format!("Search failed: {error}");
            }
        }
    }

    pub(crate) fn open_selected_repo(&mut self) {
        let Some(repo) = self.selected_repo().cloned() else {
            self.status = "No repository selected.".to_string();
            return;
        };

        self.status = "Loading...".to_string();
        let mut branches = match self.github.fetch_branches(&repo.full_name) {
            Ok(items) if !items.is_empty() => items,
            Ok(_) | Err(_) => vec![repo.default_branch.clone()],
        };
        branches.sort();
        branches.dedup();

        self.branches = branches;
        self.current_repo = Some(repo.clone());
        self.selected_branch = self
            .branches
            .iter()
            .position(|branch| {
                self.account
                    .last_branch_by_repo
                    .get(&repo.full_name)
                    .is_some_and(|saved| saved == branch)
            })
            .or_else(|| self.branches.iter().position(|b| b == &repo.default_branch))
            .unwrap_or(0);

        self.load_tree_for_current_branch();
        self.focus = Focus::Tree;
    }

    pub(crate) fn load_tree_for_current_branch(&mut self) {
        let Some(repo) = self.current_repo.as_ref() else {
            self.status = "No repository loaded.".to_string();
            return;
        };
        let full_name = repo.full_name.clone();
        let branch = self.selected_branch_name();

        self.status = "Loading...".to_string();
        match self.github.fetch_repo_tree(&full_name, &branch) {
            Ok(tree) => {
                self.reset_tree(tree);
                self.preview_title = format!("{full_name} / {branch}");
                self.preview_lines = vec![Line::from(
                    "Repository loaded. Use arrows and Enter to preview files.",
                )];
                self.account
                    .last_branch_by_repo
                    .insert(full_name.clone(), branch.clone());
                let _ = self.account.save();
                self.status = format!(
                    "Loaded branch {branch}. Tree entries: {} (showing {}). Press c to clone.",
                    self.tree_all.len(),
                    self.tree_visible_limit
                );
                self.preview_scroll = 0;
                self.current_preview_path = None;
                self.tree_text_mode = false;
            }
            Err(error) => {
                self.status = format!("Unable to open branch {branch}: {error}");
            }
        }
    }

    pub(crate) fn preview_selected_file(&mut self) {
        let (Some(repo), Some(node)) = (self.current_repo.as_ref(), self.selected_node()) else {
            return;
        };
        let full_name = repo.full_name.clone();
        let branch = self.selected_branch_name();
        let node_path = node.path.clone();
        let node_is_dir = node.is_dir;

        if node_is_dir {
            self.preview_title = format!("{}/{}", full_name, node_path);
            self.preview_lines = vec![Line::from("Directory selected. Choose a file to preview.")];
            self.preview_scroll = 0;
            self.current_preview_path = None;
            self.tree_text_mode = false;
            return;
        }

        if let Some(content) = self.preview_cache.get(&full_name, &branch, &node_path) {
            self.preview_title = format!("{}/{}", full_name, node_path);
            self.preview_scroll = 0;
            self.current_preview_path = Some(node_path.clone());
            self.tree_text_mode = false;
            match String::from_utf8(content) {
                Ok(content_str) => {
                    self.preview_lines = highlight_content(&content_str, &node_path, 300);
                    self.status = format!("Preview loaded from cache for {}", node_path);
                }
                Err(_) => {
                    self.preview_lines = vec![Line::from("Binary file. Use 'd' to download.")];
                    self.current_preview_path = None;
                    self.status = format!("Binary file in cache: {}", node_path);
                }
            }
            return;
        }

        self.status = "Loading...".to_string();
        match self.github.fetch_file_content(&full_name, &node_path) {
            Ok(bytes) => {
                self.preview_cache
                    .put(&full_name, &branch, &node_path, &bytes, None);
                self.preview_title = format!("{}/{}", full_name, node_path);
                self.preview_scroll = 0;
                self.current_preview_path = Some(node_path.clone());
                self.tree_text_mode = false;
                match String::from_utf8(bytes) {
                    Ok(content) => {
                        self.preview_lines = highlight_content(&content, &node_path, 300);
                        self.status = format!("Preview loaded for {}", node_path);
                    }
                    Err(_) => {
                        self.preview_lines = vec![Line::from("Binary file. Use 'd' to download.")];
                        self.current_preview_path = None;
                        self.status = format!("Binary file: {}", node_path);
                    }
                }
            }
            Err(error) => {
                self.status = format!("Preview failed: {error}");
            }
        }
    }

    pub(crate) fn clone_current_repo(&mut self) {
        let Some(repo) = self.current_repo.as_ref() else {
            self.status = "Open a repository before cloning.".to_string();
            return;
        };

        let destination = self.clone_path_input.trim();
        if destination.is_empty() {
            self.status = "Destination path cannot be empty.".to_string();
            return;
        }

        let destination_path = PathBuf::from(destination);
        if !destination_path.exists()
            && let Err(error) = std::fs::create_dir_all(&destination_path)
        {
            self.status = format!(
                "Cannot create destination path {}: {error}",
                destination_path.display()
            );
            return;
        }

        let output = Command::new("git")
            .arg("clone")
            .arg(&repo.clone_url)
            .current_dir(&destination_path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                self.status = format!("Repository cloned to {}", destination_path.display());
                self.focus = Focus::Tree;
                self.account.preferred_clone_dir = destination_path.display().to_string();
                let _ = self.account.save();
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                self.status = format!("git clone failed: {}", stderr.trim());
            }
            Err(error) => {
                self.status = format!("Unable to run git clone: {error}");
            }
        }
    }

    pub(crate) fn save_token_from_input_str(&mut self, token: String) {
        let token_trimmed = token.trim().to_string();
        if token_trimmed.is_empty() {
            self.status = "Token is empty.".to_string();
            return;
        }

        match auth::save_token(&token_trimmed).and_then(|_| {
            crate::provider::create_provider(
                crate::provider::ProviderKind::GitHub,
                Some(&token_trimmed),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))
        }) {
            Ok(provider) => {
                self.github = provider;
                self.auth_user = self.github.fetch_authenticated_user().ok().flatten();
                self.status = match self.auth_user.as_ref() {
                    Some(login) => format!("Token saved and validated as {login}."),
                    None => "Token saved, but validation failed.".to_string(),
                };
                self.focus = Focus::Repos;
            }
            Err(error) => {
                self.status = format!("Token save failed: {error}");
            }
        }
    }

    pub(crate) fn run_oauth_quick_check(&mut self) {
        match oauth::oauth_status_cli() {
            Ok(()) => {
                self.status =
                    "OAuth status printed in terminal. For login use: gitnapse auth oauth login"
                        .to_string();
            }
            Err(error) => {
                self.status = format!("OAuth status check failed: {error}");
            }
        }
    }

    pub(crate) fn run_oauth_login_flow(&mut self, client_id: Option<String>) {
        self.status = "Starting OAuth device flow...".to_string();

        // Temporarily leave TUI mode to let user interact with OAuth instructions in terminal.
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let guard = TerminalGuard;

        let oauth_result =
            oauth::oauth_device_login_cli(client_id, vec!["read:user".to_string()], 900);

        drop(guard);

        match oauth_result {
            Ok(()) => {
                if let Ok(token) = auth::load_token()
                    && let Ok(provider) = crate::provider::create_provider(
                        crate::provider::ProviderKind::GitHub,
                        token.as_deref(),
                    )
                {
                    self.github = provider;
                    self.auth_user = self.github.fetch_authenticated_user().ok().flatten();
                }
                self.status = "OAuth login completed and session saved.".to_string();
            }
            Err(error) => {
                self.status = format!("OAuth login failed: {error}");
            }
        }

        self.focus = if self.current_repo.is_some() {
            Focus::Tree
        } else {
            Focus::Repos
        };
    }

    pub(crate) fn toggle_tree_view(&mut self) {
        if self.current_repo.is_none() {
            self.status = "Open a repository first.".to_string();
            return;
        }
        self.tree_text_mode = !self.tree_text_mode;
        self.preview_scroll = 0;
        if self.tree_text_mode {
            let branch = self.selected_branch_name();
            self.preview_title = format!(
                "tree {} [{}]",
                self.current_repo
                    .as_ref()
                    .map(|r| r.full_name.clone())
                    .unwrap_or_default(),
                branch
            );
            self.preview_lines = self
                .tree_all
                .iter()
                .map(|node| {
                    let indent = "  ".repeat(node.depth.min(20));
                    let icon = if node.is_dir { "[D]" } else { "[F]" };
                    Line::from(format!("{indent}{icon} {}", node.path))
                })
                .collect();
            self.current_preview_path = None;
            self.focus = Focus::Preview;
            self.status = "Tree view enabled in preview pane.".to_string();
        } else {
            self.preview_title = "Preview".to_string();
            self.preview_lines = vec![Line::from(
                "Tree preview disabled. Select a file and press Enter to preview.",
            )];
            self.focus = Focus::Tree;
            self.status = "Tree view disabled.".to_string();
        }
    }
}
