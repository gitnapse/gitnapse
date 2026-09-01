use super::screens::Screen;
use super::{App, Focus, theme};
use crate::config::KeybindingsConfig;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use secrecy::ExposeSecret;

#[derive(Debug, Clone, Copy)]
pub struct PaneAreas {
    pub repo_or_tree: Rect,
    pub preview: Option<Rect>,
}

pub fn compute_panes(area: Rect, has_repo_open: bool, kb: &KeybindingsConfig) -> Option<PaneAreas> {
    let nav_lines = theme::nav_hint_lines(kb, usize::from(area.width.saturating_sub(4)));
    let nav_height = (nav_lines.len() as u16).saturating_add(2).max(3);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(nav_height),
        ])
        .split(area);

    let main = chunks.get(1).copied()?;
    if !has_repo_open {
        return Some(PaneAreas {
            repo_or_tree: main,
            preview: None,
        });
    }

    if main.width >= 120 {
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(main);
        return Some(PaneAreas {
            repo_or_tree: sections[0],
            preview: Some(sections[1]),
        });
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main);
    Some(PaneAreas {
        repo_or_tree: sections[0],
        preview: Some(sections[1]),
    })
}

pub fn render(frame: &mut Frame<'_>, app: &mut App) {
    let nav_lines = theme::nav_hint_lines(
        &app.keybindings,
        usize::from(frame.area().width.saturating_sub(4)),
    );
    let nav_height = (nav_lines.len() as u16).saturating_add(2).max(3);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(nav_height),
        ])
        .split(frame.area());

    let search_label = if app.focus == Focus::Search {
        app.input_buffer.clone()
    } else {
        app.search.query.clone()
    };
    let search_block = Paragraph::new(search_label)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search ( / then Enter )"),
        )
        .style(if app.focus == Focus::Search {
            theme::selection_style(1)
        } else {
            Style::default()
        });
    frame.render_widget(search_block, chunks[0]);

    if app.current_repo.is_some() {
        render_repo_view(frame, app, chunks[1]);
    } else {
        app.search.render(frame, app, chunks[1]);
    }

    let status_title = match app.github.rate_limit_remaining() {
        Some(n) => format!("Status — API: {n} remaining"),
        None => "Status".to_string(),
    };
    let status = Paragraph::new(app.status.clone())
        .block(Block::default().borders(Borders::ALL).title(status_title));
    frame.render_widget(status, chunks[2]);

    let nav = Paragraph::new(nav_lines)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .wrap(Wrap { trim: false });
    frame.render_widget(nav, chunks[3]);

    if app.focus == Focus::ClonePath {
        let area = centered_rect(frame.area(), 70, 20);
        frame.render_widget(Clear, area);
        let modal = Paragraph::new(app.clone_path_input.clone())
            .block(
                Block::default()
                    .title(
                        "Clone Destination Path (Type path, Del clear, Enter confirm, Esc cancel)",
                    )
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(modal, area);
    }

    if app.focus == Focus::TokenInput {
        let area = centered_rect(frame.area(), 70, 20);
        frame.render_widget(Clear, area);
        let masked = "*".repeat(app.token_buffer.expose_secret().chars().count());
        let modal = Paragraph::new(masked).block(
            Block::default()
                .title("GitHub Token (masked, Enter save, Esc cancel)")
                .borders(Borders::ALL),
        );
        frame.render_widget(modal, area);
    }

    if app.focus == Focus::OAuthClientIdInput {
        let area = centered_rect(frame.area(), 75, 20);
        frame.render_widget(Clear, area);
        let modal = Paragraph::new(app.oauth_client_id_input.clone()).block(
            Block::default()
                .title("OAuth Client ID (optional; Enter start, Del clear, Esc cancel)")
                .borders(Borders::ALL),
        );
        frame.render_widget(modal, area);
    }

    if app.focus == Focus::BranchPicker {
        let area = centered_rect(frame.area(), 60, 45);
        frame.render_widget(Clear, area);
        let items = app
            .branches
            .iter()
            .enumerate()
            .map(|(index, branch)| {
                let style = if index == app.selected_branch {
                    theme::selection_style(index)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(format!(" {branch}"), style)))
            })
            .collect::<Vec<_>>();
        let list = List::new(items).block(
            Block::default()
                .title("Branch Selector (Up/Down + Enter)")
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }

    if app.focus == Focus::TreeSearch {
        let area = centered_rect(frame.area(), 60, 20);
        frame.render_widget(Clear, area);
        let modal = Paragraph::new(app.tree_search_input.clone()).block(
            Block::default()
                .title("Find File (fuzzy match — type, Enter search, Esc cancel)")
                .borders(Borders::ALL),
        );
        frame.render_widget(modal, area);
    }

    if app.focus == Focus::DownloadPath {
        let area = centered_rect(frame.area(), 70, 20);
        frame.render_widget(Clear, area);
        let modal = Paragraph::new(app.download_path_input.clone()).block(
            Block::default()
                .title("Download Current File (type path, Del clear, Enter save, Esc cancel)")
                .borders(Borders::ALL),
        );
        frame.render_widget(modal, area);
    }

    if app.command_palette.visible {
        let area = centered_rect(frame.area(), 60, 60);
        app.command_palette.render(frame, area);
    }
}

fn render_repo_view(frame: &mut Frame<'_>, app: &mut App, area: Rect) {
    let show_side_preview = area.width >= 120;

    if show_side_preview {
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);
        app.preview_viewport_rows = usize::from(sections[1].height.saturating_sub(2)).max(1);
        render_tree(frame, &*app, sections[0]);
        render_preview(frame, &*app, sections[1]);
    } else {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);
        app.preview_viewport_rows = usize::from(sections[1].height.saturating_sub(2)).max(1);
        render_tree(frame, &*app, sections[0]);
        render_preview(frame, &*app, sections[1]);
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
            theme::selection_style(3).fg(Color::White)
        } else {
            Style::default()
        });
    frame.render_widget(List::new(items).block(block), area);
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
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if app.focus == Focus::Preview {
                    theme::selection_style(4)
                } else {
                    Style::default()
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
