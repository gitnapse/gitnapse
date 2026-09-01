use std::collections::HashSet;

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{App, Focus, theme};
use crate::models::RepoSummary;

/// Repository search + list state (Fase A2).
///
/// Owns everything about the search view: the query, pagination, the loaded
/// repositories and the current/multi selection. Rendering and small selection
/// ops live here; network calls and tree resets stay in the `App` orchestrator.
#[derive(Debug, Clone, Default)]
pub struct SearchScreen {
    pub query: String,
    pub page: u32,
    pub per_page: u8,
    pub repos: Vec<RepoSummary>,
    pub selected: usize,
    pub multi_selected: HashSet<usize>,
}

impl SearchScreen {
    pub fn selected_repo(&self) -> Option<&RepoSummary> {
        self.repos.get(self.selected)
    }

    /// Replace the loaded list with fresh results and reset the selection.
    pub fn apply_results(&mut self, items: Vec<RepoSummary>) {
        self.repos = items;
        self.selected = 0;
    }

    /// Move the list selection by `delta` (-1 / +1).
    pub fn move_selection(&mut self, delta: isize) {
        if self.repos.is_empty() {
            return;
        }
        if delta < 0 {
            self.selected = self.selected.saturating_sub(delta.unsigned_abs());
        } else {
            self.selected = (self.selected + delta as usize).min(self.repos.len() - 1);
        }
    }

    /// Toggle multi-select for the current row. Returns the new selection count.
    pub fn toggle_multi_select(&mut self) -> usize {
        if self.repos.is_empty() {
            return 0;
        }
        let idx = self.selected;
        if !self.multi_selected.remove(&idx) {
            self.multi_selected.insert(idx);
        }
        self.multi_selected.len()
    }

    /// Visible window (selected-centered) for rendering and mouse hit-testing.
    pub fn window(&self, area_height: u16) -> (usize, usize) {
        let viewport_rows = usize::from(area_height.saturating_sub(2)).max(1);
        let max_start = self.repos.len().saturating_sub(viewport_rows);
        let start = self
            .selected
            .saturating_sub(viewport_rows / 2)
            .min(max_start);
        let end = (start + viewport_rows).min(self.repos.len());
        (start, end)
    }

    /// Render the repository list pane (or the info overlay when open).
    pub fn render(&self, frame: &mut Frame<'_>, app: &App, area: Rect) {
        if app.show_info {
            let version = env!("CARGO_PKG_VERSION");
            let info = vec![
                Line::from(Span::raw("")),
                Line::from(Span::raw("")),
                Line::from(Span::raw("        GitNapse")),
                Line::from(Span::raw("")),
                Line::from(Span::raw(format!("        Version {version}"))),
                Line::from(Span::raw("")),
                Line::from(Span::raw("        https://github.com/xscriptor/gitnapse")),
                Line::from(Span::raw("")),
                Line::from(Span::raw("        Author: xscriptor")),
                Line::from(Span::raw("")),
                Line::from(Span::raw("")),
                Line::from(Span::raw("        Press / to search repositories")),
            ];
            let block = Block::default().title("Info").borders(Borders::ALL);
            let paragraph = Paragraph::new(info)
                .block(block)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, area);
            return;
        }

        if self.repos.is_empty() {
            return;
        }

        let (start, end) = self.window(area.height);
        let items = self.repos[start..end]
            .iter()
            .enumerate()
            .map(|(index, repo)| {
                let absolute = start + index;
                let marker = if absolute == self.selected { ">" } else { " " };
                let select = if self.multi_selected.contains(&absolute) {
                    "[*]"
                } else {
                    "[ ]"
                };
                let desc = repo.description.as_deref().unwrap_or("No description");
                let lang = repo.language.as_deref().unwrap_or("unknown");
                let line = format!(
                    "{marker} {select} {}/{} | ★{} | {} | {}",
                    repo.owner.login, repo.name, repo.stargazers_count, lang, desc
                );
                let style = if absolute == self.selected {
                    theme::selection_style(absolute)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(line, style)))
            })
            .collect::<Vec<_>>();

        let block = Block::default()
            .title(format!(
                "Repositories (page {} | per_page {} | shown {}-{} / {} | [ prev ] next)",
                self.page,
                self.per_page,
                if self.repos.is_empty() { 0 } else { start + 1 },
                end,
                self.repos.len()
            ))
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::Repos {
                theme::selection_style(2).fg(Color::White)
            } else {
                Style::default()
            });
        frame.render_widget(
            List::new(items).block(block).style(theme::text_style()),
            area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(name: &str) -> RepoSummary {
        RepoSummary {
            name: name.into(),
            full_name: format!("o/{name}"),
            description: None,
            stargazers_count: 0,
            language: None,
            clone_url: String::new(),
            owner: crate::models::RepoOwner { login: "o".into() },
            default_branch: "main".into(),
        }
    }

    fn screen(n: usize) -> SearchScreen {
        SearchScreen {
            repos: (0..n).map(|i| repo(&format!("r{i}"))).collect(),
            page: 1,
            per_page: 30,
            ..SearchScreen::default()
        }
    }

    #[test]
    fn apply_results_resets_selection() {
        let mut s = screen(5);
        s.selected = 3;
        s.apply_results((0..2).map(|i| repo(&format!("x{i}"))).collect());
        assert_eq!(s.repos.len(), 2);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn move_selection_clamps() {
        let mut s = screen(3);
        s.move_selection(1);
        assert_eq!(s.selected, 1);
        s.move_selection(10);
        assert_eq!(s.selected, 2);
        s.move_selection(-10);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn move_selection_noop_when_empty() {
        let mut s = SearchScreen::default();
        s.move_selection(1);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn toggle_multi_select_counts() {
        let mut s = screen(3);
        assert_eq!(s.toggle_multi_select(), 1);
        s.move_selection(1);
        assert_eq!(s.toggle_multi_select(), 2);
        assert_eq!(s.toggle_multi_select(), 1);
    }

    #[test]
    fn window_is_selected_centered() {
        let mut s = screen(100);
        s.selected = 50;
        let (start, end) = s.window(10);
        assert_eq!(start, 46);
        assert_eq!(end, 54);
    }
}
