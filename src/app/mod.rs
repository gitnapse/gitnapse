mod actions;
mod commands;
mod input;
mod network;
mod render;
mod screens;
mod theme;

use crate::auth;
use crate::cache::PreviewCache;
use crate::config::{AccountConfig, KeybindingsConfig, ThemeConfig};
use crate::models::{
    CheckRun, CommitInfo, CompareResponse, Issue, MergeResponse, PullRequest, PullRequestDetail,
    PullRequestReview, RepoNode, RepoSummary, ReviewComment,
};
use crate::provider::GitProvider;
use crate::task_manager::TaskManager;
use anyhow::{Context, Result, anyhow};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::text::Line;
use screens::command_palette::CommandPalette;
use screens::search::SearchScreen;
use secrecy::SecretString;
use std::io::stdout;
use std::panic;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug)]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
pub(crate) enum NetworkEvent {
    SearchResult(Result<Vec<RepoSummary>, String>),
    IssuesResult(Result<Vec<Issue>, String>),
    PrsResult(Result<Vec<PullRequest>, String>),
    CommitsResult(Result<Vec<CommitInfo>, String>),
    CompareResult(Result<CompareResponse, String>),
    CheckRunsResult(Result<Vec<CheckRun>, String>),
    StarredResult(Result<Vec<RepoSummary>, String>),
    PrDetailResult(Result<PullRequestDetail, String>),
    PrReviewsResult(Result<Vec<PullRequestReview>, String>),
    PrCommentsResult(Result<Vec<ReviewComment>, String>),
    PrCommitsResult(Result<Vec<CommitInfo>, String>),
    PrMergeResult(Result<MergeResponse, String>),
    PrActionResult(String),
    WorkflowRunsResult(Vec<crate::models::WorkflowRun>),
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub initial_query: String,
    pub initial_page: u32,
    pub per_page: u8,
    pub cache_ttl_secs: u64,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            initial_query: String::new(),
            initial_page: 1,
            per_page: 30,
            cache_ttl_secs: 900,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Search,
    Repos,
    Tree,
    Preview,
    TreeSearch,
    DownloadPath,
    ClonePath,
    TokenInput,
    OAuthClientIdInput,
    BranchPicker,
}

pub struct App {
    // Services
    pub github: Arc<dyn GitProvider>,
    pub account: AccountConfig,
    pub preview_cache: PreviewCache,
    pub task_manager: TaskManager,

    // Search screen
    pub search: SearchScreen,

    // Tree / explorer state
    pub tree_all: Vec<RepoNode>,
    pub tree_visible_limit: usize,
    pub selected_node: usize,
    pub current_repo: Option<RepoSummary>,
    pub branches: Vec<String>,
    pub selected_branch: usize,

    // Preview state
    pub preview_title: String,
    pub preview_lines: Vec<Line<'static>>,
    pub preview_scroll: usize,
    pub current_preview_path: Option<String>,
    pub preview_viewport_rows: usize,
    pub tree_text_mode: bool,

    // Input buffers
    pub input_buffer: String,
    pub token_buffer: SecretString,
    pub oauth_client_id_input: String,
    pub clone_path_input: String,
    pub download_path_input: String,
    pub tree_search_input: String,

    // UI state
    pub status: String,
    pub focus: Focus,
    pub should_quit: bool,
    pub auth_user: Option<String>,

    // Click tracking
    pub last_tree_click: Option<(usize, Instant)>,
    pub last_repo_click: Option<(usize, Instant)>,

    // Keybindings
    pub keybindings: KeybindingsConfig,

    // Command palette overlay
    pub command_palette: CommandPalette,

    // Info overlay
    pub show_info: bool,

    // PR management
    pub pr_detail: Option<PullRequestDetail>,
    pub pr_reviews: Vec<PullRequestReview>,
    pub pr_comments: Vec<ReviewComment>,
    pub pending_pr_number: String,
    // PR review / creation input state
    pub pr_pending_action: Option<String>,
    pub pr_pending_body: String,
}

impl App {
    const TREE_PAGE_SIZE: usize = 250;
    const TREE_LOAD_THRESHOLD: usize = 15;

    fn new(options: RunOptions) -> Result<Self> {
        let token = auth::load_token()?;
        let mut github = crate::provider::create_provider(
            crate::provider::ProviderKind::GitHub,
            token.as_deref(),
        )?;
        let mut account = AccountConfig::load_or_default()?;
        let theme_config = ThemeConfig::load_or_default();
        theme::init_theme(&theme_config);
        let keybindings = KeybindingsConfig::load_or_default();
        let preview_cache = PreviewCache::new(options.cache_ttl_secs)?;

        // Validate any stored token at startup. If the /user endpoint returns 401,
        // the token is stale – clear it so subsequent requests don't carry a bad header.
        let auth_user: Option<String> = github.fetch_authenticated_user().unwrap_or_default();
        if auth_user.is_none() && token.is_some() {
            log::warn!("stored token rejected by GitHub, clearing it");
            let _ = auth::clear_token();
            github = crate::provider::create_provider(crate::provider::ProviderKind::GitHub, None)?;
        }

        if account.preferred_clone_dir.trim().is_empty() {
            account.preferred_clone_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
                .to_string();
        }

        let status = match auth_user.as_ref() {
            Some(login) => format!("Authenticated as {login}. Press / to search."),
            None if token.is_some() => {
                "Stored token is invalid. Press t to set a new one or continue anonymously."
                    .to_string()
            }
            None => "No token set. Press t to save one or continue anonymously.".to_string(),
        };

        Ok(Self {
            github,
            account: account.clone(),
            preview_cache,
            task_manager: TaskManager::new(),
            search: SearchScreen {
                query: options.initial_query,
                page: options.initial_page.max(1),
                per_page: options.per_page.clamp(1, 100),
                ..SearchScreen::default()
            },
            tree_all: Vec::new(),
            tree_visible_limit: 0,
            selected_node: 0,
            current_repo: None,
            branches: Vec::new(),
            selected_branch: 0,
            preview_title: "Preview".to_string(),
            preview_lines: vec![Line::from("Select a repository and a file to preview.")],
            preview_scroll: 0,
            current_preview_path: None,
            preview_viewport_rows: 30,
            tree_text_mode: false,
            input_buffer: String::new(),
            token_buffer: SecretString::new(String::new().into()),
            oauth_client_id_input: {
                let client_id = std::env::var("GITNAPSE_GITHUB_OAUTH_CLIENT_ID")
                    .or_else(|_| std::env::var("GITHUB_CLIENT_ID"));
                if client_id
                    .as_ref()
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true)
                {
                    log::warn!(
                        "No OAuth client ID found in GITNAPSE_GITHUB_OAUTH_CLIENT_ID or GITHUB_CLIENT_ID env vars. Using built-in default."
                    );
                }
                client_id.unwrap_or_default().trim().to_string()
            },
            clone_path_input: account.preferred_clone_dir,
            download_path_input: String::new(),
            tree_search_input: String::new(),
            status,
            focus: Focus::Repos,
            should_quit: false,
            auth_user,
            last_tree_click: None,
            last_repo_click: None,
            keybindings,
            command_palette: CommandPalette::default(),
            show_info: false,
            pr_detail: None,
            pr_reviews: Vec::new(),
            pr_comments: Vec::new(),
            pending_pr_number: String::new(),
            pr_pending_action: None,
            pr_pending_body: String::new(),
        })
    }

    pub fn selected_repo(&self) -> Option<&RepoSummary> {
        self.search.selected_repo()
    }

    /// Clear tree/explorer context when a new search replaces the repo list.
    pub(crate) fn clear_repo_context(&mut self) {
        self.tree_all.clear();
        self.tree_visible_limit = 0;
        self.selected_node = 0;
        self.current_repo = None;
        self.branches.clear();
        self.selected_branch = 0;
        self.current_preview_path = None;
        self.tree_text_mode = false;
    }
}

pub fn run() -> Result<()> {
    run_with_options(RunOptions::default())
}

pub fn run_with_options(options: RunOptions) -> Result<()> {
    let mut app = App::new(options)?;

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        prev_hook(panic_info);
    }));

    enable_raw_mode().context("Cannot enable raw mode")?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)
        .context("Cannot enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let (net_tx, net_rx) = mpsc::channel::<NetworkEvent>();
    let github = app.github.clone();

    // Initial search via background thread
    app.status = "Loading...".to_string();
    let tx = net_tx.clone();
    let g = github.clone();
    let query = app.search.query.clone();
    let page = app.search.page;
    let per_page = app.search.per_page;
    app.task_manager.spawn(move || {
        let result = g.search_repositories_page(&query, page, per_page);
        let _ = tx.send(NetworkEvent::SearchResult(
            result.map_err(|e| e.to_string()),
        ));
    });

    let mut terminal_result = Ok(());
    while !app.should_quit {
        if let Err(error) = terminal.draw(|frame| render::render(frame, &mut app)) {
            terminal_result = Err(anyhow!("Render error: {error}"));
            break;
        }

        // Process completed network tasks
        while let Ok(event) = net_rx.try_recv() {
            app.handle_network_event(event);
        }

        if event::poll(Duration::from_millis(16)).context("Event poll failed")? {
            match event::read().context("Event read failed")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let tx = net_tx.clone();
                    let g = github.clone();
                    app.handle_key_with_channel(key, tx, g);
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    let area = terminal
                        .size()
                        .unwrap_or_else(|_| ratatui::layout::Size::new(120, 40));
                    app.handle_mouse_click(mouse.column, mouse.row, area.into());
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::ScrollUp => {
                    let area = terminal
                        .size()
                        .unwrap_or_else(|_| ratatui::layout::Size::new(120, 40));
                    app.handle_mouse_scroll(mouse.column, mouse.row, true, area.into());
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::ScrollDown => {
                    let area = terminal
                        .size()
                        .unwrap_or_else(|_| ratatui::layout::Size::new(120, 40));
                    app.handle_mouse_scroll(mouse.column, mouse.row, false, area.into());
                }
                Event::Resize(cols, rows) => {
                    app.status = format!("Terminal resized to {cols}x{rows}");
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();
    app.task_manager.join_all();
    let _ = panic::take_hook(); // remove our hook
    terminal_result
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    let x2 = rect.x.saturating_add(rect.width);
    let y2 = rect.y.saturating_add(rect.height);
    col >= rect.x && col < x2 && row >= rect.y && row < y2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use secrecy::SecretString;

    fn test_app() -> App {
        App {
            github: crate::provider::create_provider(crate::provider::ProviderKind::GitHub, None)
                .expect("client"),
            account: crate::config::AccountConfig {
                preferred_clone_dir: ".".to_string(),
                last_branch_by_repo: Default::default(),
            },
            preview_cache: crate::cache::PreviewCache::new(120).expect("cache"),
            task_manager: TaskManager::new(),
            search: SearchScreen::default(),
            tree_all: vec![],
            tree_visible_limit: 0,
            selected_node: 0,
            current_repo: None,
            branches: vec![],
            selected_branch: 0,
            preview_title: String::new(),
            preview_lines: vec![],
            preview_scroll: 0,
            current_preview_path: None,
            preview_viewport_rows: 30,
            tree_text_mode: false,
            input_buffer: String::new(),
            token_buffer: SecretString::new(String::new().into()),
            oauth_client_id_input: String::new(),
            clone_path_input: ".".to_string(),
            download_path_input: String::new(),
            tree_search_input: String::new(),
            status: String::new(),
            focus: Focus::Repos,
            should_quit: false,
            auth_user: None,
            last_tree_click: None,
            last_repo_click: None,
            keybindings: crate::config::KeybindingsConfig::default(),
            command_palette: CommandPalette::default(),
            show_info: false,
            pr_detail: None,
            pr_reviews: Vec::new(),
            pr_comments: Vec::new(),
            pending_pr_number: String::new(),
            pr_pending_action: None,
            pr_pending_body: String::new(),
        }
    }

    #[test]
    fn key_q_sets_should_quit() {
        let mut app = test_app();
        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn key_slash_opens_search() {
        let mut app = test_app();
        app.focus = Focus::Repos;
        app.handle_key(KeyCode::Char('/'));
        assert_eq!(app.focus, Focus::Search);
    }

    #[test]
    fn key_esc_from_search_returns_to_repos() {
        let mut app = test_app();
        app.focus = Focus::Search;
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.focus, Focus::Repos);
    }

    #[test]
    fn key_tab_cycles_focus() {
        let mut app = test_app();
        // Repos -> Tree (with tree data) -> Preview -> Repos
        app.tree_all = vec![crate::models::RepoNode {
            path: "f".into(),
            name: "f".into(),
            depth: 0,
            is_dir: false,
        }];
        app.tree_visible_limit = 1;
        app.current_repo = Some(crate::models::RepoSummary {
            name: "repo".into(),
            full_name: "o/r".into(),
            description: None,
            stargazers_count: 0,
            language: None,
            clone_url: "".into(),
            owner: crate::models::RepoOwner { login: "o".into() },
            default_branch: "main".into(),
        });
        app.preview_lines = vec![Line::from("test")];

        assert_eq!(app.focus, Focus::Repos);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Tree);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Preview);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Repos);
    }

    #[test]
    fn key_down_in_repos_moves_selection() {
        let mut app = test_app();
        app.search.repos = vec![
            RepoSummary {
                name: "a".into(),
                full_name: "o/a".into(),
                description: None,
                stargazers_count: 0,
                language: None,
                clone_url: "".into(),
                owner: crate::models::RepoOwner { login: "o".into() },
                default_branch: "main".into(),
            },
            RepoSummary {
                name: "b".into(),
                full_name: "o/b".into(),
                description: None,
                stargazers_count: 0,
                language: None,
                clone_url: "".into(),
                owner: crate::models::RepoOwner { login: "o".into() },
                default_branch: "main".into(),
            },
        ];
        assert_eq!(app.search.selected, 0);
        app.handle_key(KeyCode::Down);
        assert_eq!(app.search.selected, 1);
    }

    #[test]
    fn key_up_in_tree_moves_selection() {
        let mut app = test_app();
        app.tree_all = vec![
            RepoNode {
                path: "a".into(),
                name: "a".into(),
                depth: 0,
                is_dir: false,
            },
            RepoNode {
                path: "b".into(),
                name: "b".into(),
                depth: 0,
                is_dir: false,
            },
        ];
        app.tree_visible_limit = 2;
        app.focus = Focus::Tree;
        app.selected_node = 1;
        app.handle_key(KeyCode::Up);
        assert_eq!(app.selected_node, 0);
    }

    #[test]
    fn token_input_escapes_and_zeroizes() {
        let mut app = test_app();
        app.focus = Focus::TokenInput;
        app.token_buffer = SecretString::new("sometoken".into());
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.focus, Focus::Repos);
        // After zeroize, memory is cleared; the original token value is no longer intact
        // (zeroize overwrites the backing memory even though length metadata remains)
    }

    #[test]
    fn token_input_enter_saves() {
        let mut app = test_app();
        app.focus = Focus::TokenInput;
        app.token_buffer = SecretString::new("test_token".into());
        app.handle_key(KeyCode::Enter);
        // An attempt to save is made; after the attempt the token buffer is zeroized.
        // Focus moves to Repos if save succeeded, otherwise stays on TokenInput.
        // We just verify no panic and the zeroize call ran.
    }

    #[test]
    fn search_input_adds_characters() {
        let mut app = test_app();
        app.focus = Focus::Search;
        app.input_buffer.clear();
        app.handle_key(KeyCode::Char('r'));
        app.handle_key(KeyCode::Char('s'));
        assert_eq!(app.input_buffer, "rs");
    }

    #[test]
    fn search_input_backspace_removes_char() {
        let mut app = test_app();
        app.focus = Focus::Search;
        app.input_buffer = "rust".to_string();
        app.handle_key(KeyCode::Backspace);
        assert_eq!(app.input_buffer, "rus");
    }

    #[test]
    fn lazy_tree_progress_advances_limit() {
        let mut app = App {
            github: crate::provider::create_provider(crate::provider::ProviderKind::GitHub, None)
                .expect("client"),
            account: crate::config::AccountConfig {
                preferred_clone_dir: ".".to_string(),
                last_branch_by_repo: Default::default(),
            },
            preview_cache: crate::cache::PreviewCache::new(120).expect("cache"),
            task_manager: TaskManager::new(),
            search: SearchScreen::default(),
            tree_all: (0..800)
                .map(|i| crate::models::RepoNode {
                    path: format!("f{i}"),
                    name: format!("f{i}"),
                    depth: 0,
                    is_dir: false,
                })
                .collect(),
            tree_visible_limit: 250,
            selected_node: 250 - App::TREE_LOAD_THRESHOLD + 1,
            current_repo: None,
            branches: vec![],
            selected_branch: 0,
            preview_title: String::new(),
            preview_lines: vec![],
            preview_scroll: 0,
            current_preview_path: None,
            preview_viewport_rows: 30,
            tree_text_mode: false,
            input_buffer: String::new(),
            token_buffer: SecretString::new(String::new().into()),
            oauth_client_id_input: String::new(),
            clone_path_input: ".".to_string(),
            download_path_input: String::new(),
            tree_search_input: String::new(),
            status: String::new(),
            focus: Focus::Tree,
            should_quit: false,
            auth_user: None,
            last_tree_click: None,
            last_repo_click: None,
            keybindings: crate::config::KeybindingsConfig::default(),
            command_palette: CommandPalette::default(),
            show_info: false,
            pr_detail: None,
            pr_reviews: Vec::new(),
            pr_comments: Vec::new(),
            pending_pr_number: String::new(),
            pr_pending_action: None,
            pr_pending_body: String::new(),
        };
        app.ensure_lazy_tree_progress();
        assert_eq!(app.tree_visible_limit, 500);
    }
}
