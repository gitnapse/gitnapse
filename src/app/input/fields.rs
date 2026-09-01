use crate::app::{App, Focus};
use anyhow::Context;
use crossterm::event::KeyCode;
use secrecy::{SecretString, zeroize::Zeroize};
use std::path::PathBuf;

fn fuzzy_match(needle: &str, haystack: &str) -> bool {
    let needle = needle.chars().collect::<Vec<_>>();
    if needle.is_empty() {
        return true;
    }
    let mut ni = 0;
    for c in haystack.chars() {
        if c.eq_ignore_ascii_case(&needle[ni]) {
            ni += 1;
            if ni == needle.len() {
                return true;
            }
        }
    }
    false
}

impl App {
    pub(super) fn handle_search_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.focus = Focus::Repos;
        } else if self.keybindings.matches_key("enter", &code) {
            self.search.query = self.input_buffer.trim().to_string();
            self.search.page = 1;
            self.focus = Focus::Repos;
            self.search();
        } else if self.keybindings.matches_key("backspace", &code) {
            self.input_buffer.pop();
        } else if let KeyCode::Char(ch) = code {
            self.input_buffer.push(ch);
        }
    }

    pub(super) fn handle_tree_search_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.tree_search_input.clear();
            self.focus = Focus::Tree;
        } else if self.keybindings.matches_key("enter", &code) {
            let needle = self.tree_search_input.trim().to_ascii_lowercase();
            self.focus = Focus::Tree;
            if needle.is_empty() {
                self.status = "Search term is empty.".to_string();
                return;
            }
            if let Some((idx, _)) = self
                .tree_all
                .iter()
                .enumerate()
                .find(|(_, n)| fuzzy_match(&needle, &n.path))
            {
                self.selected_node = idx;
                self.ensure_lazy_tree_progress();
                self.status = format!(
                    "Found file match for \"{}\".",
                    self.tree_search_input.trim()
                );
            } else {
                self.status = format!("No file matches \"{}\".", self.tree_search_input.trim());
            }
        } else if self.keybindings.matches_key("backspace", &code) {
            self.tree_search_input.pop();
        } else if let KeyCode::Char(ch) = code {
            self.tree_search_input.push(ch);
        }
    }

    pub(super) fn handle_download_path_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.focus = Focus::Preview;
        } else if self.keybindings.matches_key("enter", &code) {
            let Some(repo) = self.current_repo.as_ref() else {
                self.status = "No repository loaded.".to_string();
                self.focus = Focus::Tree;
                return;
            };
            let Some(file_path) = self.current_preview_path.as_ref() else {
                self.status = "Preview a file first before downloading.".to_string();
                self.focus = Focus::Tree;
                return;
            };

            let out = self.download_path_input.trim();
            if out.is_empty() {
                self.status = "Download path cannot be empty.".to_string();
                return;
            }
            let out_path = PathBuf::from(out);
            if let Some(parent) = out_path.parent()
                && !parent.as_os_str().is_empty()
                && let Err(error) = std::fs::create_dir_all(parent)
            {
                self.status = format!("Cannot create parent folder: {error}");
                return;
            }
            let branch = self.selected_branch_name();
            match self
                .github
                .fetch_file_content_by_ref(&repo.full_name, file_path, &branch)
                .and_then(|content| {
                    std::fs::write(&out_path, content)
                        .map_err(anyhow::Error::from)
                        .context("Cannot write downloaded file")
                }) {
                Ok(()) => {
                    self.status = format!(
                        "Downloaded {}:{} -> {}",
                        repo.full_name,
                        file_path,
                        out_path.display()
                    );
                    self.focus = Focus::Preview;
                }
                Err(error) => {
                    self.status = format!("Download failed: {error}");
                }
            }
        } else if self.keybindings.matches_key("delete", &code) {
            self.download_path_input.clear();
        } else if self.keybindings.matches_key("backspace", &code) {
            self.download_path_input.pop();
        } else if let KeyCode::Char(ch) = code {
            self.download_path_input.push(ch);
        }
    }

    pub(super) fn handle_clone_path_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.focus = Focus::Tree;
        } else if self.keybindings.matches_key("enter", &code) {
            self.clone_current_repo();
        } else if self.keybindings.matches_key("delete", &code) {
            self.clone_path_input.clear();
        } else if self.keybindings.matches_key("backspace", &code) {
            self.clone_path_input.pop();
        } else if let KeyCode::Char(ch) = code {
            self.clone_path_input.push(ch);
        }
    }

    pub(super) fn handle_token_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.token_buffer.zeroize();
            self.input_buffer.clear();
            self.focus = Focus::Repos;
        } else if self.keybindings.matches_key("enter", &code) {
            let token = std::mem::take(&mut self.input_buffer);
            self.token_buffer = SecretString::new(token.clone().into());
            self.save_token_from_input_str(token);
            self.token_buffer.zeroize();
        } else if self.keybindings.matches_key("backspace", &code) {
            self.input_buffer.pop();
        } else if let KeyCode::Char(ch) = code {
            self.input_buffer.push(ch);
        }
    }

    pub(super) fn handle_oauth_client_id_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.focus = if self.current_repo.is_some() {
                Focus::Tree
            } else {
                Focus::Repos
            };
        } else if self.keybindings.matches_key("enter", &code) {
            let client_id = if self.oauth_client_id_input.trim().is_empty() {
                None
            } else {
                Some(self.oauth_client_id_input.trim().to_string())
            };
            self.run_oauth_login_flow(client_id);
        } else if self.keybindings.matches_key("delete", &code) {
            self.oauth_client_id_input.clear();
        } else if self.keybindings.matches_key("backspace", &code) {
            self.oauth_client_id_input.pop();
        } else if let KeyCode::Char(ch) = code {
            self.oauth_client_id_input.push(ch);
        }
    }

    pub(super) fn handle_branch_picker_input(&mut self, code: KeyCode) {
        if self.keybindings.matches_key("escape", &code) {
            self.focus = Focus::Tree;
        } else if self.keybindings.matches_key("scroll_up", &code) {
            self.selected_branch = self.selected_branch.saturating_sub(1);
        } else if self.keybindings.matches_key("scroll_down", &code) {
            if !self.branches.is_empty() {
                self.selected_branch = (self.selected_branch + 1).min(self.branches.len() - 1);
            }
        } else if self.keybindings.matches_key("enter", &code) {
            self.load_tree_for_current_branch();
            self.focus = Focus::Tree;
        }
    }
}
