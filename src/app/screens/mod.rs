pub mod command_palette;
pub mod search;
pub mod tree;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::config::KeybindingsConfig;

/// A screen/overlay owns its UI state and exposes a uniform interface so the
/// app event loop and renderer stay small as features grow.
///
/// `handle_key` borrows only the config it needs and returns an `Outcome` that
/// the `App` executes afterwards. This keeps screens decoupled from `App`'s
/// fields (no `&mut App` borrow conflicts) and makes them unit-testable.
pub trait Screen {
    /// What the screen wants the `App` to do after handling a key event.
    type Outcome;

    /// Handle a key event against the current screen state.
    fn handle_key(&mut self, keybindings: &KeybindingsConfig, event: KeyEvent) -> Self::Outcome;

    /// Render the screen into `area`.
    fn render(&self, frame: &mut Frame<'_>, area: Rect);
}
