use crate::app::{self, App, Focus};
use ratatui::layout::Rect;
use std::time::{Duration, Instant};

impl App {
    pub(crate) fn handle_mouse_click(&mut self, col: u16, row: u16, terminal_area: Rect) {
        let Some(panes) = app::render::compute_panes(
            terminal_area,
            self.current_repo.is_some(),
            &self.keybindings,
        ) else {
            return;
        };
        if app::contains(panes.repo_or_tree, col, row) {
            if self.current_repo.is_some() {
                self.focus = Focus::Tree;
                let content_row = row.saturating_sub(panes.repo_or_tree.y.saturating_add(1));
                let (start, end) = self.tree_window(panes.repo_or_tree.height);
                let idx = start + usize::from(content_row);
                if idx < end && idx < self.tree_all.len() {
                    self.selected_node = idx;
                    self.ensure_lazy_tree_progress();
                    if self.tree_all.get(idx).map(|n| !n.is_dir).unwrap_or(false)
                        && self.is_double_click_tree(idx)
                    {
                        self.preview_selected_file();
                        self.focus = Focus::Preview;
                    }
                }
            } else {
                self.focus = Focus::Repos;
                let content_row = row.saturating_sub(panes.repo_or_tree.y.saturating_add(1));
                let (start, end) = self.repo_window(panes.repo_or_tree.height);
                let idx = start + usize::from(content_row);
                if idx < end && idx < self.search.repos.len() {
                    self.search.selected = idx;
                    if self.is_double_click_repo(idx) {
                        self.open_selected_repo();
                    }
                }
            }
            return;
        }
        if let Some(preview_area) = panes.preview
            && app::contains(preview_area, col, row)
        {
            self.focus = Focus::Preview;
        }
    }

    pub(crate) fn handle_mouse_scroll(
        &mut self,
        col: u16,
        row: u16,
        up: bool,
        terminal_area: Rect,
    ) {
        let Some(panes) = app::render::compute_panes(
            terminal_area,
            self.current_repo.is_some(),
            &self.keybindings,
        ) else {
            return;
        };
        if app::contains(panes.repo_or_tree, col, row) {
            if self.current_repo.is_some() && !self.tree_all.is_empty() {
                self.focus = Focus::Tree;
                if up {
                    self.selected_node = self.selected_node.saturating_sub(1);
                } else {
                    self.selected_node =
                        (self.selected_node + 1).min(self.tree_all.len().saturating_sub(1));
                    self.ensure_lazy_tree_progress();
                }
            } else if !self.search.repos.is_empty() {
                self.focus = Focus::Repos;
                if up {
                    self.search.move_selection(-1);
                } else {
                    self.search.move_selection(1);
                }
            }
            return;
        }
        if let Some(preview_area) = panes.preview
            && app::contains(preview_area, col, row)
        {
            self.focus = Focus::Preview;
            if up {
                self.scroll_preview_up(3);
            } else {
                self.scroll_preview_down(
                    3,
                    usize::from(preview_area.height.saturating_sub(2)).max(1),
                );
            }
        }
    }

    fn is_double_click_tree(&mut self, idx: usize) -> bool {
        let now = Instant::now();
        let is_double = self
            .last_tree_click
            .map(|(last_idx, last_at)| {
                last_idx == idx && now.duration_since(last_at) <= Duration::from_millis(450)
            })
            .unwrap_or(false);
        self.last_tree_click = Some((idx, now));
        is_double
    }

    fn is_double_click_repo(&mut self, idx: usize) -> bool {
        let now = Instant::now();
        let is_double = self
            .last_repo_click
            .map(|(last_idx, last_at)| {
                last_idx == idx && now.duration_since(last_at) <= Duration::from_millis(450)
            })
            .unwrap_or(false);
        self.last_repo_click = Some((idx, now));
        is_double
    }
}
