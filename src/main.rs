use anyhow::Result;
use clap::{Parser, Subcommand};

use gitnapse::{app, auth, cli, oauth};

#[derive(Debug, Parser)]
#[command(
    name = "gitnapse",
    version,
    about = "Terminal GitHub repository explorer"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run interactive terminal UI
    Run(cli::RunArgs),
    /// Download one file from a GitHub repository (curl/wget-like)
    DownloadFile(cli::DownloadFileArgs),
    /// Manage GitHub token authentication
    Auth {
        #[command(subcommand)]
        action: cli::AuthAction,
    },
    /// Clone a repository (via API + git)
    Clone(cli::CloneArgs),
    /// Stage (with -a) and commit changes
    Commit(cli::CommitArgs),
    /// Push commits to remote
    Push(cli::PushArgs),
    /// Pull changes from remote (with --rebase)
    Pull(cli::PullArgs),
    /// Fetch from remote (with --prune)
    Fetch(cli::FetchArgs),
    /// Switch branches or restore files
    Checkout(cli::CheckoutArgs),
    /// Show working tree diff
    Diff(cli::DiffArgs),
    /// Stash changes
    Stash {
        #[command(subcommand)]
        action: cli::StashAction,
    },
    /// Manage tags
    Tag {
        #[command(subcommand)]
        action: cli::TagAction,
    },
    /// Show working tree status
    Status,
    /// Show commit log (default: 20 entries)
    Log(cli::LogArgs),
    /// List branches
    Branch,
    /// Reset current HEAD
    Reset(cli::ResetArgs),
    /// Manage pull requests via GitHub API
    Pr {
        #[command(subcommand)]
        action: cli::PrAction,
    },
    /// Manage issues via GitHub API
    Issue {
        #[command(subcommand)]
        action: cli::IssueAction,
    },
    /// Show CI status for a repository
    Ci(cli::CiArgs),
    /// Compare two branches
    Compare(cli::CompareArgs),
    /// Manage remotes
    Remote {
        #[command(subcommand)]
        action: cli::RemoteAction,
    },
    /// Manage git config
    Config {
        #[command(subcommand)]
        action: cli::ConfigAction,
    },
    /// Merge a branch into current
    Merge(cli::MergeArgs),
    /// Manage releases via GitHub API
    Release {
        #[command(subcommand)]
        action: cli::ReleaseAction,
    },
    /// Manage repositories via GitHub API
    Repo {
        #[command(subcommand)]
        action: cli::RepoAction,
    },
    /// Search repositories on GitHub
    Search(cli::SearchArgs),
    /// List, install or remove themes from the registry
    Theme {
        #[command(subcommand)]
        action: cli::ThemeAction,
    },
}

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    gitnapse::runtime::ensure_crypto_provider();
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(cmd: Option<Command>) -> Result<()> {
    use Command::{
        Auth, Branch, Checkout, Ci, Clone, Commit, Compare, Config, Diff, DownloadFile, Fetch,
        Issue, Log, Merge, Pr, Pull, Push, Release, Remote, Repo, Reset, Run, Search, Stash,
        Status, Tag, Theme,
    };
    match cmd {
        Some(Run(args)) => app::run_with_options(args.into()),
        Some(DownloadFile(args)) => {
            cli::download_file(&args.repo, &args.path, args.r#ref.as_deref(), &args.out)
        }
        Some(Clone(args)) => cli::clone_repo(&args.repo, args.dir.as_deref()),
        Some(Commit(args)) => cli::commit(&args.message, args.all),
        Some(Push(args)) => cli::push(args.remote.as_deref(), args.branch.as_deref(), args.force),
        Some(Pull(args)) => cli::pull(args.remote.as_deref(), args.branch.as_deref(), args.rebase),
        Some(Fetch(args)) => cli::fetch(args.prune),
        Some(Checkout(args)) => cli::checkout(&args.branch, args.create),
        Some(Diff(args)) => cli::diff(args.staged, args.path.as_deref()),
        Some(Stash { action }) => dispatch_stash(action),
        Some(Tag { action }) => dispatch_tag(action),
        Some(Status) => cli::status(),
        Some(Log(args)) => cli::log_lines(args.count),
        Some(Branch) => cli::branch(),
        Some(Reset(args)) => cli::reset(args.target.as_deref(), args.hard),
        Some(Pr { action }) => dispatch_pr(action),
        Some(Issue { action }) => dispatch_issue(action),
        Some(Ci(args)) => cli::ci_status(&args.repo, args.branch.as_deref(), args.workflows),
        Some(Compare(args)) => cli::compare(&args.repo, &args.base, &args.head),
        Some(Remote { action }) => dispatch_remote(action),
        Some(Config { action }) => dispatch_config(action),
        Some(Merge(args)) => cli::merge(&args.branch),
        Some(Release { action }) => dispatch_release(action),
        Some(Repo { action }) => dispatch_repo(action),
        Some(Search(args)) => cli::search(&args.query),
        Some(Theme { action }) => dispatch_theme(action),
        Some(Auth { action }) => dispatch_auth(action),
        None => app::run(),
    }
}

fn dispatch_stash(action: cli::StashAction) -> Result<()> {
    use cli::StashAction::{List, Pop, Push};
    match action {
        Push { message } => cli::stash_push(message.as_deref()),
        Pop => cli::stash_pop(),
        List => cli::stash_list(),
    }
}

fn dispatch_tag(action: cli::TagAction) -> Result<()> {
    use cli::TagAction::{Create, Delete, List};
    match action {
        List { pattern } => cli::tag_list(pattern.as_deref()),
        Create {
            name,
            message,
            target,
        } => cli::tag_create(&name, message.as_deref(), target.as_deref()),
        Delete { name } => cli::tag_delete(&name),
    }
}

fn dispatch_pr(action: cli::PrAction) -> Result<()> {
    use cli::PrAction::{Create, List, Merge};
    match action {
        List(a) => cli::pr_list(&a.repo, &a.state),
        Create(a) => cli::pr_create(&a.repo, &a.title, &a.head, &a.base, a.body.as_deref()),
        Merge(a) => cli::pr_merge(&a.repo, a.number, a.method.as_deref()),
    }
}

fn dispatch_issue(action: cli::IssueAction) -> Result<()> {
    use cli::IssueAction::{Close, Create, List};
    match action {
        List(a) => cli::issue_list(&a.repo, &a.state),
        Create(a) => cli::issue_create(&a.repo, &a.title, a.body.as_deref()),
        Close(a) => cli::issue_close(&a.repo, a.number),
    }
}

fn dispatch_remote(action: cli::RemoteAction) -> Result<()> {
    use cli::RemoteAction::{Add, List, Remove, Rename};
    match action {
        List => cli::remote_list(),
        Add(a) => cli::remote_add(&a.name, &a.url),
        Remove(a) => cli::remote_remove(&a.name),
        Rename(a) => cli::remote_rename(&a.old, &a.new),
    }
}

fn dispatch_config(action: cli::ConfigAction) -> Result<()> {
    use cli::ConfigAction::{Get, List, Set};
    match action {
        Get(a) => cli::config_get(&a.key),
        Set(a) => cli::config_set(&a.key, &a.value),
        List => cli::config_list(),
    }
}

fn dispatch_release(action: cli::ReleaseAction) -> Result<()> {
    use cli::ReleaseAction::{Create, List};
    match action {
        List(a) => cli::release_list(&a.repo),
        Create(a) => cli::release_create(
            &a.repo,
            &a.tag_name,
            a.name.as_deref(),
            a.body.as_deref(),
            a.prerelease,
        ),
    }
}

fn dispatch_repo(action: cli::RepoAction) -> Result<()> {
    use cli::RepoAction::Create;
    match action {
        Create(a) => cli::repo_create(&a.name, a.description.as_deref(), a.private),
    }
}

fn dispatch_theme(action: cli::ThemeAction) -> Result<()> {
    use cli::ThemeAction::{Install, List, Uninstall};
    use gitnapse::registry;
    match action {
        List(args) => {
            let local = registry::list_installed()?;
            println!("Installed themes ({}):", local.len());
            for name in &local {
                println!("  {name}");
            }
            if args.remote {
                println!("\nRegistry: {}", registry::index_url());
                let remote = registry::list_remote()?;
                for entry in remote {
                    let mark = if local.iter().any(|l| l.eq_ignore_ascii_case(&entry.name)) {
                        "[installed] "
                    } else {
                        ""
                    };
                    let shade = if entry.dark { "dark" } else { "light" };
                    let desc = entry.description.as_deref().unwrap_or("");
                    println!("  {mark}{} ({shade}) {}", entry.name, desc);
                }
            } else {
                println!(
                    "Use --remote to query the registry at {}",
                    registry::index_url()
                );
            }
            Ok(())
        }
        Install(args) => {
            let entry = registry::install_theme(&args.name)?;
            println!("Installed theme '{}' from the registry.", entry.name);
            println!("Select it with theme_name in theme.jsonc or via the app theme picker.");
            Ok(())
        }
        Uninstall(args) => {
            if registry::uninstall_theme(&args.name)? {
                println!("Removed theme '{}'.", args.name);
            } else {
                println!("Theme '{}' is not installed locally.", args.name);
            }
            Ok(())
        }
    }
}

fn dispatch_auth(action: cli::AuthAction) -> Result<()> {
    use cli::AuthAction::{Clear, Oauth, Set, Status};
    match action {
        Set { token } => auth::set_token_cli(token),
        Clear => auth::clear_token_cli(),
        Status => auth::status_cli(),
        Oauth { action } => dispatch_oauth(action),
    }
}

fn dispatch_oauth(action: cli::OauthAction) -> Result<()> {
    use cli::OauthAction::{Login, Status};
    match action {
        Login {
            client_id,
            scope,
            timeout_secs,
        } => oauth::oauth_device_login_cli(client_id, scope, timeout_secs),
        Status => oauth::oauth_status_cli(),
    }
}
