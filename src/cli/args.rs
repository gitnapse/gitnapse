use clap::{Args, Subcommand};
use std::path::PathBuf;

// ── Top-level Args ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    /// Initial search query
    #[arg(long, default_value = "")]
    pub query: String,
    /// Search results page to load
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    /// Results per page (max 100)
    #[arg(long, default_value_t = 30)]
    pub per_page: u8,
    /// Preview cache TTL in seconds
    #[arg(long, default_value_t = 900)]
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Args)]
pub struct DownloadFileArgs {
    /// Repository in owner/name form
    #[arg(long)]
    pub repo: String,
    /// Path of the file inside the repository
    #[arg(long)]
    pub path: String,
    /// Branch/tag/commit (default: HEAD)
    #[arg(long)]
    pub r#ref: Option<String>,
    /// Destination local file path
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct CloneArgs {
    /// owner/repo[:branch] or a full git URL
    pub repo: String,
    /// Destination directory (default: repository name in the current dir)
    #[arg(long)]
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct CommitArgs {
    /// Commit message
    #[arg(short = 'm')]
    pub message: String,
    /// Stage all changes first (git add -A)
    #[arg(short = 'a')]
    pub all: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PushArgs {
    /// Remote name (default: origin)
    pub remote: Option<String>,
    /// Branch to push (default: current branch)
    pub branch: Option<String>,
    /// Force push with --force-with-lease
    #[arg(long = "force-with-lease")]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PullArgs {
    /// Remote name (default: origin)
    pub remote: Option<String>,
    /// Branch to pull (default: current branch)
    pub branch: Option<String>,
    /// Rebase instead of merge
    #[arg(long)]
    pub rebase: bool,
}

#[derive(Debug, Clone, Args)]
pub struct FetchArgs {
    /// Remove stale remote-tracking branches
    #[arg(long)]
    pub prune: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CheckoutArgs {
    /// Branch to switch to
    pub branch: String,
    /// Create the branch before switching
    #[arg(short = 'b')]
    pub create: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DiffArgs {
    /// Show staged changes instead of unstaged
    #[arg(long)]
    pub staged: bool,
    /// Restrict the diff to a single path
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct LogArgs {
    /// Number of commits to show
    #[arg(short = 'n', default_value_t = 20)]
    pub count: usize,
}

#[derive(Debug, Clone, Args)]
pub struct ResetArgs {
    /// Ref to reset to (default: HEAD)
    pub target: Option<String>,
    /// Discard working tree changes
    #[arg(long)]
    pub hard: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CiArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// Branch to inspect (default: main)
    #[arg(short = 'b', long)]
    pub branch: Option<String>,
    /// Also list workflow runs
    #[arg(short = 'w', long)]
    pub workflows: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CompareArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// Base branch
    pub base: String,
    /// Head branch
    pub head: String,
}

// ── PR subcommand args ──────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct PrListArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// State filter: open, closed or all
    #[arg(short = 's', long, default_value = "open")]
    pub state: String,
}

#[derive(Debug, Clone, Args)]
pub struct PrCreateArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// Pull request title
    #[arg(short = 't', long)]
    pub title: String,
    /// Source branch (head)
    #[arg(short = 'H', long)]
    pub head: String,
    /// Target branch (base)
    #[arg(short = 'B', long)]
    pub base: String,
    /// Body / description
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PrMergeArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// Pull request number
    #[arg(short = 'n', long)]
    pub number: u64,
    /// Merge method: merge, squash or rebase
    #[arg(short = 'm', long)]
    pub method: Option<String>,
}

// ── Issue subcommand args ───────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct IssueListArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// State filter: open, closed or all
    #[arg(short = 's', long, default_value = "open")]
    pub state: String,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCreateArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// Issue title
    #[arg(short = 't', long)]
    pub title: String,
    /// Body / description
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCloseArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
    pub repo: String,
    /// Issue number
    #[arg(short = 'n', long)]
    pub number: u64,
}

// ── Remote subcommand args ──────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct RemoteAddArgs {
    /// Remote name (e.g. upstream)
    pub name: String,
    /// Remote URL
    pub url: String,
}

#[derive(Debug, Clone, Args)]
pub struct RemoteRemoveArgs {
    /// Remote name
    pub name: String,
}

#[derive(Debug, Clone, Args)]
pub struct RemoteRenameArgs {
    /// Current remote name
    pub old: String,
    /// New remote name
    pub new: String,
}

// ── Config subcommand args ──────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct ConfigGetArgs {
    /// Git config key (e.g. user.name)
    pub key: String,
}

#[derive(Debug, Clone, Args)]
pub struct ConfigSetArgs {
    /// Git config key (e.g. user.name)
    pub key: String,
    /// Value to set
    pub value: String,
}

// ── Merge arg ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct MergeArgs {
    /// Branch to merge into the current branch
    pub branch: String,
}

// ── Release subcommand args ─────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct ReleaseListArgs {
    /// Repository in owner/name form (auto-detected inside a clone)
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
    /// Theme name as shown by `theme list` or `theme list --remote`
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
    /// Stash local changes (optionally with a description)
    Push {
        /// Stash description (e.g. "WIP")
        #[arg(short = 'm')]
        message: Option<String>,
    },
    /// Restore and drop the topmost stash
    Pop,
    /// List stashed entries
    List,
}

#[derive(Debug, Subcommand)]
pub enum TagAction {
    /// List tags, optionally filtered by a glob pattern
    List {
        /// Glob pattern to filter tags (e.g. "v*")
        pattern: Option<String>,
    },
    /// Create a new tag
    Create {
        /// Tag name (e.g. v1.0.0)
        name: String,
        /// Create an annotated tag with this message
        #[arg(short = 'm')]
        message: Option<String>,
        /// Commit/branch/ref the tag points at (default: HEAD)
        #[arg(long)]
        target: Option<String>,
    },
    /// Delete a tag locally and on the remote
    Delete {
        /// Tag name
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PrAction {
    /// List pull requests of a repository
    List(PrListArgs),
    /// Create a pull request
    Create(PrCreateArgs),
    /// Merge a pull request
    Merge(PrMergeArgs),
}

#[derive(Debug, Subcommand)]
pub enum IssueAction {
    /// List issues of a repository
    List(IssueListArgs),
    /// Create an issue
    Create(IssueCreateArgs),
    /// Close an issue
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
    /// Store a GitHub token (prompts for it if --token is omitted)
    Set {
        /// Token to store
        #[arg(long)]
        token: Option<String>,
    },
    /// Remove the stored token
    Clear,
    /// Show which token source is in use
    Status,
    /// OAuth device-flow management
    Oauth {
        #[command(subcommand)]
        action: OauthAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum OauthAction {
    /// Start the OAuth device flow (opens the browser and stores the session)
    Login {
        /// OAuth client id
        #[arg(long)]
        client_id: Option<String>,
        /// OAuth scopes, comma separated (e.g. read:user,repo)
        #[arg(long = "scope", value_delimiter = ',')]
        scope: Vec<String>,
        /// Seconds to wait for the user to authorize the device
        #[arg(long, default_value_t = 900)]
        timeout_secs: u64,
    },
    /// Show the stored OAuth session status
    Status,
}

// ── From impls ──────────────────────────────────────────────────────────

#[cfg(feature = "tui")]
impl From<RunArgs> for crate::app::RunOptions {
    fn from(value: RunArgs) -> Self {
        Self {
            initial_query: value.query,
            initial_page: value.page.max(1),
            per_page: value.per_page.clamp(1, 100),
            cache_ttl_secs: value.cache_ttl_secs.max(1),
        }
    }
}
