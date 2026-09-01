use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::{App, theme};
use crate::config::KeybindingsConfig;

use super::Screen;

/// Outcome of handling a key in the command palette: the `App` performs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteOutcome {
    None,
    ExecuteCommand(String),
    ExecutePrAction(String),
    ThemePicked(String),
}

/// Command palette overlay: owns its state and input handling.
///
/// Also doubles as the results surface for network-driven lists (issues, PRs,
/// commits, CI checks, diffs, PR detail/reviews/comments), mirroring the
/// previous `command_*` fields on `App`.
#[derive(Debug, Clone, Default)]
pub struct CommandPalette {
    pub visible: bool,
    pub input: String,
    pub cursor: usize,
    pub items: Vec<String>,
    pub filtered: Vec<String>,
    pub is_pr_action: bool,
    pub is_theme_picker: bool,
}

impl CommandPalette {
    /// Build the context-sensitive command list for the current app state
    /// (was `App::build_command_list`).
    pub fn command_list(app: &App) -> Vec<String> {
        let mut commands = vec![
            "Search Repositories".to_string(),
            "List Starred Repos".to_string(),
        ];
        if app.current_repo.is_some() {
            commands.push("Switch Branch".to_string());
            commands.push("Find File".to_string());
            commands.push("Clone Repository".to_string());
            commands.push("Download Current File".to_string());
            commands.push("Toggle Tree View".to_string());
            commands.push("List Issues".to_string());
            commands.push("List Pull Requests".to_string());
            commands.push("View Recent Commits".to_string());
            commands.push("View CI Status".to_string());
            commands.push("View Workflow Runs".to_string());
            commands.push("View PR Detail".to_string());
            commands.push("Create Pull Request".to_string());
            commands.push("Compare Branches".to_string());
        }
        if !app.search.multi_selected.is_empty() {
            commands.push(format!(
                "Clone {} Selected Repos",
                app.search.multi_selected.len()
            ));
        }
        commands.push("Show Info".to_string());
        commands.push("Change Theme".to_string());
        commands.push("Set Token".to_string());
        commands.push("Quit".to_string());
        commands
    }

    /// Present a set of items as selectable palette entries (network results,
    /// PR detail, reviews, comments, commits). `pr_action` marks them as PR
    /// actions so Enter routes through the PR action handler.
    pub fn show(&mut self, items: Vec<String>, pr_action: bool) {
        self.items = items;
        self.filtered.clear();
        self.cursor = 0;
        self.input.clear();
        self.is_pr_action = pr_action;
        self.is_theme_picker = false;
        self.visible = true;
    }

    /// Enter theme-picker mode: the items are theme names and Enter applies
    /// the selected theme.
    pub fn enter_theme_picker(&mut self, themes: Vec<String>) {
        self.show(themes, false);
        self.is_theme_picker = true;
    }

    fn selected(&self) -> Option<String> {
        let list = if self.input.is_empty() {
            &self.items
        } else {
            &self.filtered
        };
        list.get(self.cursor).cloned()
    }

    fn update_filter(&mut self) {
        if self.input.is_empty() {
            self.filtered.clear();
            return;
        }
        let lower = self.input.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .filter(|item| item.to_lowercase().contains(&lower))
            .cloned()
            .collect();
        let count = self.filtered.len();
        if count > 0 {
            self.cursor = self.cursor.min(count - 1);
        } else {
            self.cursor = 0;
        }
    }
}

impl Screen for CommandPalette {
    type Outcome = PaletteOutcome;

    fn handle_key(&mut self, keybindings: &KeybindingsConfig, event: KeyEvent) -> PaletteOutcome {
        let code = event.code;
        if keybindings.matches_key("escape", &code) {
            self.visible = false;
            PaletteOutcome::None
        } else if keybindings.matches_key("enter", &code) {
            let selected = self.selected();
            if self.is_theme_picker {
                self.visible = false;
                self.is_theme_picker = false;
                match selected {
                    Some(theme_name) => PaletteOutcome::ThemePicked(theme_name),
                    None => PaletteOutcome::None,
                }
            } else if self.is_pr_action {
                self.visible = false;
                self.is_pr_action = false;
                match selected {
                    Some(action) => PaletteOutcome::ExecutePrAction(action),
                    None => PaletteOutcome::None,
                }
            } else {
                self.visible = false;
                match selected {
                    Some(cmd) => PaletteOutcome::ExecuteCommand(cmd),
                    None => PaletteOutcome::None,
                }
            }
        } else if keybindings.matches_key("scroll_up", &code) {
            let count = if self.input.is_empty() {
                self.items.len()
            } else {
                self.filtered.len()
            };
            if count > 0 {
                self.cursor = self.cursor.saturating_sub(1);
            }
            PaletteOutcome::None
        } else if keybindings.matches_key("scroll_down", &code) {
            let count = if self.input.is_empty() {
                self.items.len()
            } else {
                self.filtered.len()
            };
            if count > 0 {
                self.cursor = (self.cursor + 1).min(count - 1);
            }
            PaletteOutcome::None
        } else if keybindings.matches_key("backspace", &code) {
            self.input.pop();
            self.update_filter();
            PaletteOutcome::None
        } else if let KeyCode::Char(ch) = code {
            self.input.push(ch);
            self.update_filter();
            PaletteOutcome::None
        } else {
            PaletteOutcome::None
        }
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);

        let items = {
            let list = if self.input.is_empty() {
                &self.items
            } else {
                &self.filtered
            };
            if list.is_empty() {
                vec![ListItem::new(Line::from(" No matching commands"))]
            } else {
                list.iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        let style = if i == self.cursor {
                            theme::selection_style(i)
                        } else {
                            Style::default()
                        };
                        ListItem::new(Line::from(Span::styled(format!(" {}", cmd), style)))
                    })
                    .collect()
            }
        };

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        let search_input = Paragraph::new(self.input.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Palette (Ctrl+P, type to filter, Enter to execute)"),
        );
        frame.render_widget(search_input, inner[0]);

        let list_widget = List::new(items).block(Block::default().borders(Borders::NONE));
        frame.render_widget(list_widget, inner[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    fn palette() -> CommandPalette {
        CommandPalette {
            items: vec!["a".into(), "ab".into(), "b".into()],
            ..CommandPalette::default()
        }
    }

    #[test]
    fn default_is_closed() {
        let p = CommandPalette::default();
        assert!(!p.visible);
        assert!(p.items.is_empty());
    }

    #[test]
    fn show_resets_state() {
        let mut p = palette();
        p.input.push('x');
        p.cursor = 2;
        p.is_pr_action = true;
        p.show(vec!["one".into()], true);
        assert!(p.visible);
        assert!(p.input.is_empty());
        assert_eq!(p.cursor, 0);
        assert!(p.is_pr_action);
        assert!(!p.is_theme_picker);
    }

    #[test]
    fn filter_keeps_matches() {
        let mut p = palette();
        p.handle_key(
            &KeybindingsConfig::default(),
            KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE),
        );
        assert_eq!(p.filtered, vec!["a".to_string(), "ab".to_string()]);
        assert_eq!(p.input, "a");
    }

    #[test]
    fn enter_returns_selected_command() {
        let mut p = palette();
        let kb = KeybindingsConfig::default();
        let out = p.handle_key(
            &kb,
            KeyEvent::new(KeyCode::Char('b'), crossterm::event::KeyModifiers::NONE),
        );
        assert_eq!(out, PaletteOutcome::None);
        // "b" filters to ["ab", "b"]; cursor stays at 0 -> "ab".
        let out = p.handle_key(
            &kb,
            KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE),
        );
        assert_eq!(out, PaletteOutcome::ExecuteCommand("ab".to_string()));
        assert!(!p.visible);
    }

    #[test]
    fn enter_theme_picker_returns_theme() {
        let mut p = CommandPalette::default();
        p.enter_theme_picker(vec!["Berlin".into()]);
        assert!(p.is_theme_picker);
        let kb = KeybindingsConfig::default();
        let out = p.handle_key(
            &kb,
            KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE),
        );
        assert_eq!(out, PaletteOutcome::ThemePicked("Berlin".to_string()));
        assert!(!p.is_theme_picker);
    }

    #[test]
    fn escape_closes() {
        let mut p = palette();
        p.visible = true;
        let kb = KeybindingsConfig::default();
        let out = p.handle_key(
            &kb,
            KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE),
        );
        assert_eq!(out, PaletteOutcome::None);
        assert!(!p.visible);
    }
}
