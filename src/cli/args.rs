use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::app;

// ── Top-level Args ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    #[arg(long, default_value = "")]
    pub query: String,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long, default_value_t = 30)]
    pub per_page: u8,
    #[arg(long, default_value_t = 900)]
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Args)]
pub struct DownloadFileArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub path: String,
    #[arg(long)]
    pub r#ref: Option<String>,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct CloneArgs {
    pub repo: String,
    #[arg(long)]
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct CommitArgs {
    #[arg(short = 'm')]
    pub message: String,
    #[arg(short = 'a')]
    pub all: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PushArgs {
    pub remote: Option<String>,
    pub branch: Option<String>,
    #[arg(long = "force-with-lease")]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PullArgs {
    pub remote: Option<String>,
    pub branch: Option<String>,
    #[arg(long)]
    pub rebase: bool,
}

#[derive(Debug, Clone, Args)]
pub struct FetchArgs {
    #[arg(long)]
    pub prune: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CheckoutArgs {
    pub branch: String,
    #[arg(short = 'b')]
    pub create: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DiffArgs {
    #[arg(long)]
    pub staged: bool,
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct LogArgs {
    #[arg(short = 'n', default_value_t = 20)]
    pub count: usize,
}

#[derive(Debug, Clone, Args)]
pub struct ResetArgs {
    pub target: Option<String>,
    #[arg(long)]
    pub hard: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CiArgs {
    pub repo: String,
    #[arg(short = 'b', long)]
    pub branch: Option<String>,
    #[arg(short = 'w', long)]
    pub workflows: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CompareArgs {
    pub repo: String,
    pub base: String,
    pub head: String,
}

// ── PR subcommand args ──────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct PrListArgs {
    pub repo: String,
    #[arg(short = 's', long, default_value = "open")]
    pub state: String,
}

#[derive(Debug, Clone, Args)]
pub struct PrCreateArgs {
    pub repo: String,
    #[arg(short = 't', long)]
    pub title: String,
    #[arg(short = 'H', long)]
    pub head: String,
    #[arg(short = 'B', long)]
    pub base: String,
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PrMergeArgs {
    pub repo: String,
    #[arg(short = 'n', long)]
    pub number: u64,
    #[arg(short = 'm', long)]
    pub method: Option<String>,
}

// ── Issue subcommand args ───────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct IssueListArgs {
    pub repo: String,
    #[arg(short = 's', long, default_value = "open")]
    pub state: String,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCreateArgs {
    pub repo: String,
    #[arg(short = 't', long)]
    pub title: String,
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCloseArgs {
    pub repo: String,
    #[arg(short = 'n', long)]
    pub number: u64,
}

// ── Remote subcommand args ──────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct RemoteAddArgs {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Args)]
pub struct RemoteRemoveArgs {
    pub name: String,
}

#[derive(Debug, Clone, Args)]
pub struct RemoteRenameArgs {
    pub old: String,
    pub new: String,
}

// ── Config subcommand args ──────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct ConfigGetArgs {
    pub key: String,
}

#[derive(Debug, Clone, Args)]
pub struct ConfigSetArgs {
    pub key: String,
    pub value: String,
}

// ── Merge arg ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct MergeArgs {
    pub branch: String,
}

// ── Release subcommand args ─────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct ReleaseListArgs {
    pub repo: String,
}

#[derive(Debug, Clone, Args)]
pub struct ReleaseCreateArgs {
    pub repo: String,
    /// Git tag name for the release
    pub tag_name: String,
    /// Release title (defaults to tag_name)
    #[arg(short = 'n', long)]
    pub name: Option<String>,
    /// Release body / description
    #[arg(short = 'b', long)]
    pub body: Option<String>,
    /// Mark as pre-release
    #[arg(long)]
    pub prerelease: bool,
}

// ── Repo subcommand args ────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct RepoCreateArgs {
    /// Repository name
    pub name: String,
    /// Repository description
    #[arg(short = 'd', long)]
    pub description: Option<String>,
    /// Create as private repository
    #[arg(short = 'p', long)]
    pub private: bool,
}

// ── Search arg ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
}

// ── Theme registry ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct ThemeListArgs {
    /// Also query the remote registry (github.com/gitnapse/themes)
    #[arg(long)]
    pub remote: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ThemeInstallArgs {
    /// Theme name as listed in the registry index
    pub name: String,
}

#[derive(Debug, Clone, Args)]
pub struct ThemeUninstallArgs {
    pub name: String,
}

#[derive(Debug, Subcommand)]
pub enum ThemeAction {
    /// List installed themes (add --remote to query the registry)
    List(ThemeListArgs),
    /// Install a theme from the registry (github.com/gitnapse/themes)
    Install(ThemeInstallArgs),
    /// Remove an installed theme
    Uninstall(ThemeUninstallArgs),
}

// ── Action enums ────────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
pub enum StashAction {
    Push {
        #[arg(short = 'm')]
        message: Option<String>,
    },
    Pop,
    List,
}

#[derive(Debug, Subcommand)]
pub enum TagAction {
    List {
        pattern: Option<String>,
    },
    Create {
        name: String,
        #[arg(short = 'm')]
        message: Option<String>,
        #[arg(long)]
        target: Option<String>,
    },
    Delete {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PrAction {
    List(PrListArgs),
    Create(PrCreateArgs),
    Merge(PrMergeArgs),
}

#[derive(Debug, Subcommand)]
pub enum IssueAction {
    List(IssueListArgs),
    Create(IssueCreateArgs),
    Close(IssueCloseArgs),
}

#[derive(Debug, Subcommand)]
pub enum RemoteAction {
    /// List remotes
    List,
    /// Add a remote
    Add(RemoteAddArgs),
    /// Remove a remote
    Remove(RemoteRemoveArgs),
    /// Rename a remote
    Rename(RemoteRenameArgs),
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Get a config value
    Get(ConfigGetArgs),
    /// Set a config value
    Set(ConfigSetArgs),
    /// List all config
    List,
}

#[derive(Debug, Subcommand)]
pub enum ReleaseAction {
    /// List releases
    List(ReleaseListArgs),
    /// Create a release
    Create(ReleaseCreateArgs),
}

#[derive(Debug, Subcommand)]
pub enum RepoAction {
    /// Create a repository
    Create(RepoCreateArgs),
}

#[derive(Debug, Subcommand)]
pub enum AuthAction {
    Set {
        #[arg(long)]
        token: Option<String>,
    },
    Clear,
    Status,
    Oauth {
        #[command(subcommand)]
        action: OauthAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum OauthAction {
    Login {
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long = "scope", value_delimiter = ',')]
        scope: Vec<String>,
        #[arg(long, default_value_t = 900)]
        timeout_secs: u64,
    },
    Status,
}

// ── From impls ──────────────────────────────────────────────────────────

impl From<RunArgs> for app::RunOptions {
    fn from(value: RunArgs) -> Self {
        Self {
            initial_query: value.query,
            initial_page: value.page.max(1),
            per_page: value.per_page.clamp(1, 100),
            cache_ttl_secs: value.cache_ttl_secs.max(1),
        }
    }
}
