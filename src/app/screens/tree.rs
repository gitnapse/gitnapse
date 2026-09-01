use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{App, Focus, theme};
use crate::models::RepoNode;

// ── View state / helpers (Fase A3) ───────────────────────────────────────────

impl App {
    pub fn visible_tree(&self) -> &[RepoNode] {
        let limit = self.tree_visible_limit.min(self.tree_all.len());
        &self.tree_all[..limit]
    }

    pub(crate) fn selected_node(&self) -> Option<&RepoNode> {
        self.tree_all.get(self.selected_node)
    }

    pub(crate) fn selected_branch_name(&self) -> String {
        self.branches
            .get(self.selected_branch)
            .cloned()
            .unwrap_or_else(|| "HEAD".to_string())
    }

    pub(crate) fn ensure_lazy_tree_progress(&mut self) {
        if self.tree_visible_limit >= self.tree_all.len() {
            return;
        }
        if self.selected_node + Self::TREE_LOAD_THRESHOLD >= self.tree_visible_limit {
            self.tree_visible_limit =
                (self.tree_visible_limit + Self::TREE_PAGE_SIZE).min(self.tree_all.len());
            self.status = format!(
                "Loaded more tree entries ({}/{}).",
                self.tree_visible_limit,
                self.tree_all.len()
            );
        }
    }

    pub(crate) fn reset_tree(&mut self, nodes: Vec<RepoNode>) {
        self.tree_all = nodes;
        self.selected_node = 0;
        self.tree_visible_limit = self.tree_all.len().min(Self::TREE_PAGE_SIZE);
        self.current_preview_path = None;
        self.tree_text_mode = false;
    }

    pub(crate) fn tree_window(&self, area_height: u16) -> (usize, usize) {
        let visible = self.visible_tree();
        let viewport_rows = usize::from(area_height.saturating_sub(2)).max(1);
        let max_start = visible.len().saturating_sub(viewport_rows);
        let start = self
            .selected_node
            .saturating_sub(viewport_rows / 2)
            .min(max_start);
        let end = (start + viewport_rows).min(visible.len());
        (start, end)
    }

    pub(crate) fn scroll_preview_down(&mut self, step: usize, viewport_rows: usize) {
        let max_scroll = self.max_preview_scroll(viewport_rows);
        self.preview_scroll = (self.preview_scroll + step).min(max_scroll);
    }

    pub(crate) fn scroll_preview_up(&mut self, step: usize) {
        self.preview_scroll = self.preview_scroll.saturating_sub(step);
    }

    fn max_preview_scroll(&self, viewport_rows: usize) -> usize {
        self.preview_lines
            .len()
            .saturating_sub(viewport_rows.max(1))
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

pub(crate) fn render_repo_view(frame: &mut Frame<'_>, app: &mut App, area: Rect) {
    let show_side_preview = area.width >= 120;

    if show_side_preview {
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);
        app.preview_viewport_rows = usize::from(sections[1].height.saturating_sub(2)).max(1);
        render_tree(frame, app, sections[0]);
        render_preview(frame, app, sections[1]);
    } else {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);
        app.preview_viewport_rows = usize::from(sections[1].height.saturating_sub(2)).max(1);
        render_tree(frame, app, sections[0]);
        render_preview(frame, app, sections[1]);
    }
}

fn render_tree(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let visible = app.visible_tree();
    let viewport_rows = usize::from(area.height.saturating_sub(2)).max(1);
    let max_start = visible.len().saturating_sub(viewport_rows);
    let start = app
        .selected_node
        .saturating_sub(viewport_rows / 2)
        .min(max_start);
    let end = (start + viewport_rows).min(visible.len());

    let items = visible[start..end]
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let absolute = start + index;
            let marker = if absolute == app.selected_node {
                ">"
            } else {
                " "
            };
            let indent = "  ".repeat(entry.depth.min(20));
            let icon = if entry.is_dir { "[D]" } else { "[F]" };
            let text = format!("{marker} {indent}{icon} {}", entry.name);
            let style = if absolute == app.selected_node {
                theme::selection_style(absolute)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect::<Vec<_>>();

    let block = Block::default()
        .title(format!(
            "Explorer [{}] shown {}-{} / {} (b branches)",
            app.selected_branch_name(),
            if visible.is_empty() { 0 } else { start + 1 },
            end,
            app.tree_all.len()
        ))
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Tree {
            Style::default().fg(theme::accent3())
        } else {
            Style::default()
        });
    frame.render_widget(
        List::new(items).block(block).style(theme::text_style()),
        area,
    );
}

fn render_preview(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let viewport_rows = usize::from(area.height.saturating_sub(2)).max(1);
    let start = app
        .preview_scroll
        .min(app.preview_lines.len().saturating_sub(1));
    let end = (start + viewport_rows).min(app.preview_lines.len());
    let preview_slice = if app.preview_lines.is_empty() {
        vec![Line::from("")]
    } else {
        app.preview_lines[start..end].to_vec()
    };
    let title = format!(
        "{} ({}-{} / {})",
        app.preview_title,
        if app.preview_lines.is_empty() {
            0
        } else {
            start + 1
        },
        end,
        app.preview_lines.len()
    );

    let paragraph = Paragraph::new(preview_slice)
        .style(theme::text_style())
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if app.focus == Focus::Preview {
                    Style::default().fg(theme::accent2())
                } else {
                    Style::default()
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
