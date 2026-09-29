//! Typed local git operations.
//!
//! Every operation takes an explicit `cwd` and returns structured data; no
//! function prints to stdout or reads from stdin, so the whole module is safe
//! to call from the headless SDK, the HTTP server and the TUI.

mod parse;

use crate::cli::helpers::{
    not_a_repo_or_stderr, parse_remote_full_name, run_git_with_cwd_locale, stderr_msg, stdout_str,
};
use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Repository metadata resolved from the repository at `cwd`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoInfo {
    /// Absolute path of the repository work tree root.
    pub root: String,
    /// Directory name of the repository root.
    pub name: String,
    /// Origin remote URL, falling back to the first configured remote.
    pub remote: Option<String>,
    /// `owner/repo` parsed from the remote URL, when recognizable.
    pub full_name: Option<String>,
    /// Current branch, or `None` when the HEAD is detached.
    pub branch: Option<String>,
    /// Whether the HEAD is detached.
    pub detached: bool,
    /// Commits ahead of the upstream tracking branch.
    pub ahead: u32,
    /// Commits behind the upstream tracking branch.
    pub behind: u32,
}

/// Working tree and index state of the repository at `cwd`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    /// Current branch, or `None` when the HEAD is detached.
    pub branch: Option<String>,
    /// Whether the HEAD is detached.
    pub detached: bool,
    /// Commits ahead of the upstream tracking branch.
    pub ahead: u32,
    /// Commits behind the upstream tracking branch.
    pub behind: u32,
    /// Whether the index and working tree contain no changes.
    pub clean: bool,
    /// Changes staged in the index.
    pub staged: Vec<FileChange>,
    /// Unstaged changes in the working tree.
    pub unstaged: Vec<FileChange>,
    /// Untracked paths.
    pub untracked: Vec<String>,
    /// Paths with merge conflicts.
    pub conflicted: Vec<String>,
}

/// A single changed path, in the index or in the working tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Repository-relative path.
    pub path: String,
    /// Previous path for renames and copies.
    pub orig_path: Option<String>,
    /// Change kind: `modified`, `added`, `deleted`, `renamed`, `copied`,
    /// `typechange` or `unmerged`.
    pub kind: String,
    /// Whether the change is staged in the index.
    pub staged: bool,
}

/// A single commit in the history of the repository at `cwd`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLogEntry {
    /// Full commit hash.
    pub hash: String,
    /// Abbreviated commit hash.
    pub short: String,
    /// Author name.
    pub author_name: String,
    /// Author email.
    pub author_email: String,
    /// Author date in ISO 8601 format.
    pub date: String,
    /// First line of the commit message.
    pub subject: String,
}

/// A local branch and its upstream tracking state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    /// Short branch name.
    pub name: String,
    /// Whether this branch is the current HEAD.
    pub current: bool,
    /// Short name of the upstream tracking branch, when configured.
    pub upstream: Option<String>,
    /// Commits ahead of the upstream.
    pub ahead: u32,
    /// Commits behind the upstream.
    pub behind: u32,
}

/// A tag in the repository at `cwd`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitTag {
    /// Short tag name.
    pub name: String,
    /// Annotation message for annotated tags; `None` for lightweight tags.
    pub message: Option<String>,
    /// Tag (or commit) date in ISO 8601 format.
    pub date: Option<String>,
}

/// A configured git remote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRemote {
    /// Remote name.
    pub name: String,
    /// Fetch URL.
    pub fetch_url: String,
    /// Push URL when it differs from [`GitRemote::fetch_url`].
    pub push_url: Option<String>,
}

/// A single stash entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStashEntry {
    /// Position in the stash stack (`stash@{index}`).
    pub index: u32,
    /// Reflog selector, e.g. `stash@{0}`.
    pub name: String,
    /// Stash reflog subject.
    pub message: String,
    /// Creation date in ISO 8601 format.
    pub date: String,
}

/// Which changes [`diff`] should render.
///
/// Serializes with an internal `kind` tag, e.g.
/// `{"kind":"range","from":"main","to":"feature"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiffMode {
    /// Unstaged changes in the working tree.
    Worktree {
        /// Optional repository-relative path filter.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// Changes staged in the index.
    Staged {
        /// Optional repository-relative path filter.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// Changes between `rev` and the working tree.
    Commit {
        /// Commit-ish to diff against.
        rev: String,
        /// Optional repository-relative path filter.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// Changes between two revisions (`git diff from..to`).
    Range {
        /// Base revision.
        from: String,
        /// Head revision.
        to: String,
        /// Optional repository-relative path filter.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
}

/// A repository specification resolved into a concrete `git clone` target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloneTarget {
    /// URL passed to `git clone`: HTTPS/SSH for remotes, or any git URL
    /// (including `file://`) when the spec already was one.
    pub url: String,
    /// `owner/repo` when the spec (or URL) identifies a repository.
    pub full_name: Option<String>,
    /// Branch requested by the spec (`owner/repo:branch` / `<url>:branch`).
    pub branch: Option<String>,
}

/// Splits a clone spec into `(repo_or_url, optional branch)`.
///
/// Accepted forms: `owner/repo`, `owner/repo:branch`, full URLs
/// (`https://…`, `git@host:owner/repo.git`) and URLs with a `:branch` suffix.
pub(crate) fn parse_repo_spec(spec: &str) -> Result<(String, Option<String>)> {
    if spec.is_empty() {
        return Err(anyhow!(
            "repository specification is empty\n\
             Usage: gitnapse clone <owner/repo>[:branch] [--dir <path>]"
        ));
    }
    if spec.contains("://") || spec.contains('@') {
        if let Some(pos) = spec.rfind(':') {
            let url_part = &spec[..pos];
            let branch_part = &spec[pos + 1..];
            if !branch_part.is_empty() && !branch_part.contains('/') && !branch_part.contains('.') {
                return Ok((url_part.to_string(), Some(branch_part.to_string())));
            }
        }
        Ok((spec.to_string(), None))
    } else {
        if let Some((repo, branch)) = spec.split_once(':') {
            if repo.is_empty() {
                return Err(anyhow!(
                    "invalid repository specification '{spec}'\n\
                     Usage: gitnapse clone <owner/repo>[:branch] [--dir <path>]"
                ));
            }
            Ok((repo.to_string(), Some(branch.to_string())))
        } else {
            Ok((spec.to_string(), None))
        }
    }
}

/// Resolves a repository specification into a concrete [`CloneTarget`].
///
/// `owner/repo[:branch]` is resolved through the GitHub provider (the same
/// semantics as the CLI clone command, including `clone_url` lookup); full
/// URLs are passed through unchanged. [`CloneTarget::full_name`] is best
/// effort: the parsed `owner/repo` for recognizable URLs, the spec itself for
/// `owner/repo` forms, and `None` otherwise.
pub fn resolve_clone_target(spec: &str) -> Result<CloneTarget> {
    let (repo, branch) = parse_repo_spec(spec)?;
    if repo.contains("://") || repo.contains('@') {
        // Local/file URLs parse to `/`; only keep real `owner/repo` shapes.
        let full_name = parse_remote_full_name(&repo).filter(|name| {
            name.split_once('/')
                .is_some_and(|(owner, repo)| !owner.is_empty() && !repo.is_empty())
        });
        return Ok(CloneTarget {
            url: repo,
            full_name,
            branch,
        });
    }

    let token = crate::auth::load_token()?;
    let client =
        crate::provider::create_provider(crate::provider::ProviderKind::GitHub, token.as_deref())?;
    let info = client.fetch_repo_by_name(&repo)?;
    Ok(CloneTarget {
        url: info.clone_url,
        full_name: Some(repo),
        branch,
    })
}

fn run(cwd: &Path, args: &[&str], op: &str) -> Result<std::process::Output> {
    let output = run_git_with_cwd_locale(args, cwd)?;
    if !output.status.success() {
        bail!("{}", not_a_repo_or_stderr(&output, op));
    }
    Ok(output)
}

/// Returns the repository metadata for `cwd`.
pub fn repo_info(cwd: &Path) -> Result<GitRepoInfo> {
    let output = run(
        cwd,
        &["rev-parse", "--show-toplevel"],
        "git rev-parse failed",
    )?;
    let root = stdout_str(&output).trim().to_string();
    if root.is_empty() {
        return Err(anyhow!("{}", crate::cli::helpers::not_a_repo_msg()));
    }
    let name = Path::new(&root)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| root.clone());

    let state = status(cwd)?;
    let remote = remote_url(cwd);
    let full_name = remote.as_deref().and_then(parse_remote_full_name);

    Ok(GitRepoInfo {
        root,
        name,
        remote,
        full_name,
        branch: state.branch,
        detached: state.detached,
        ahead: state.ahead,
        behind: state.behind,
    })
}

fn remote_url(cwd: &Path) -> Option<String> {
    if let Ok(output) = crate::cli::helpers::run_git_with_cwd(&["remote", "get-url", "origin"], cwd)
        && output.status.success()
    {
        let url = stdout_str(&output).trim().to_string();
        if !url.is_empty() {
            return Some(url);
        }
    }

    let output = crate::cli::helpers::run_git_with_cwd(&["remote"], cwd).ok()?;
    if !output.status.success() {
        return None;
    }
    let first = stdout_str(&output).lines().next()?.trim().to_string();
    if first.is_empty() {
        return None;
    }
    let output =
        crate::cli::helpers::run_git_with_cwd(&["remote", "get-url", first.as_str()], cwd).ok()?;
    if !output.status.success() {
        return None;
    }
    let url = stdout_str(&output).trim().to_string();
    (!url.is_empty()).then_some(url)
}

/// Returns the working tree and index state for `cwd`.
pub fn status(cwd: &Path) -> Result<GitStatus> {
    let output = run(
        cwd,
        &["status", "--porcelain=v2", "--branch", "-z"],
        "git status failed",
    )?;
    Ok(parse::parse_status(&stdout_str(&output)))
}

/// Returns up to `limit` commits of the current branch, newest first.
///
/// A `limit` of `0` returns an empty list, and a repository without commits
/// returns an empty list instead of an error.
pub fn log(cwd: &Path, limit: usize) -> Result<Vec<GitLogEntry>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let count = limit.to_string();
    let format = format!("--format={}", parse::LOG_FORMAT);
    let output = run_git_with_cwd_locale(&["log", "-z", "-n", &count, &format], cwd)?;
    if !output.status.success() {
        let message = stderr_msg(&output);
        if message.contains("does not have any commits") || message.contains("unknown revision") {
            return Ok(Vec::new());
        }
        bail!("{}", not_a_repo_or_stderr(&output, "git log failed"));
    }
    Ok(parse::parse_log(&stdout_str(&output)))
}

/// Returns the unified diff for the requested [`DiffMode`].
pub fn diff(cwd: &Path, mode: &DiffMode) -> Result<String> {
    let mut args: Vec<String> = vec!["diff".to_string()];
    let path = match mode {
        DiffMode::Worktree { path } => path,
        DiffMode::Staged { path } => {
            args.push("--cached".to_string());
            path
        }
        DiffMode::Commit { rev, path } => {
            let rev = rev.trim();
            if rev.is_empty() {
                bail!("commit revision cannot be empty");
            }
            args.push(rev.to_string());
            path
        }
        DiffMode::Range { from, to, path } => {
            let from = from.trim();
            let to = to.trim();
            if from.is_empty() || to.is_empty() {
                bail!("diff range endpoints cannot be empty");
            }
            args.push(format!("{from}..{to}"));
            path
        }
    };
    if let Some(path) = path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        args.push("--".to_string());
        args.push(path.to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run(cwd, &refs, "git diff failed")?;
    Ok(stdout_str(&output))
}

fn require_paths(paths: &[String]) -> Result<Vec<&str>> {
    let cleaned: Vec<&str> = paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .collect();
    if cleaned.is_empty() {
        bail!("at least one path is required");
    }
    Ok(cleaned)
}

/// Stages the given paths in the index.
pub fn stage(cwd: &Path, paths: &[String]) -> Result<()> {
    let paths = require_paths(paths)?;
    let mut args = vec!["add", "--"];
    args.extend(paths);
    run(cwd, &args, "git add failed")?;
    Ok(())
}

/// Unstages the given paths, leaving the working tree untouched.
pub fn unstage(cwd: &Path, paths: &[String]) -> Result<()> {
    let paths = require_paths(paths)?;
    let mut args = vec!["restore", "--staged", "--"];
    args.extend(paths);
    run(cwd, &args, "git restore --staged failed")?;
    Ok(())
}

/// Discards working tree changes for the given tracked paths.
///
/// Untracked paths are not removed by this operation.
pub fn discard(cwd: &Path, paths: &[String]) -> Result<()> {
    let paths = require_paths(paths)?;
    let mut args = vec!["restore", "--"];
    args.extend(paths);
    run(cwd, &args, "git restore failed")?;
    Ok(())
}

/// Commits the index (optionally staging all changes first) and returns the
/// new commit hash.
pub fn commit(cwd: &Path, message: &str, all: bool) -> Result<String> {
    let message = message.trim();
    if message.is_empty() {
        bail!("commit message cannot be empty");
    }
    if all {
        run(cwd, &["add", "-A"], "git add failed")?;
    }
    run(cwd, &["commit", "-m", message], "git commit failed")?;
    let output = run(cwd, &["rev-parse", "HEAD"], "git rev-parse failed")?;
    Ok(stdout_str(&output).trim().to_string())
}

/// Pushes the current branch and returns the command's stdout.
///
/// `force` uses `--force-with-lease`, and `set_upstream` adds
/// `--set-upstream`.
pub fn push(
    cwd: &Path,
    remote: Option<&str>,
    branch: Option<&str>,
    force: bool,
    set_upstream: bool,
) -> Result<String> {
    let mut args = vec!["push"];
    if force {
        args.push("--force-with-lease");
    }
    if set_upstream {
        args.push("--set-upstream");
    }
    if let Some(remote) = remote.map(str::trim).filter(|remote| !remote.is_empty()) {
        args.push(remote);
    }
    if let Some(branch) = branch.map(str::trim).filter(|branch| !branch.is_empty()) {
        args.push(branch);
    }
    let output = run(cwd, &args, "git push failed")?;
    Ok(stdout_str(&output))
}

/// Pulls from the remote and returns the command's stdout.
pub fn pull(
    cwd: &Path,
    remote: Option<&str>,
    branch: Option<&str>,
    rebase: bool,
) -> Result<String> {
    let mut args = vec!["pull"];
    if rebase {
        args.push("--rebase");
    }
    if let Some(remote) = remote.map(str::trim).filter(|remote| !remote.is_empty()) {
        args.push(remote);
    }
    if let Some(branch) = branch.map(str::trim).filter(|branch| !branch.is_empty()) {
        args.push(branch);
    }
    let output = run(cwd, &args, "git pull failed")?;
    Ok(stdout_str(&output))
}

/// Fetches from the configured remotes and returns the command's stdout.
pub fn fetch(cwd: &Path, prune: bool) -> Result<String> {
    let mut args = vec!["fetch"];
    if prune {
        args.push("--prune");
    }
    let output = run(cwd, &args, "git fetch failed")?;
    Ok(stdout_str(&output))
}

/// Lists local branches with their upstream tracking state.
pub fn branches(cwd: &Path) -> Result<Vec<GitBranch>> {
    let format = format!("--format={}", parse::BRANCH_FORMAT);
    let output = run(
        cwd,
        &["for-each-ref", &format, "refs/heads"],
        "git branch failed",
    )?;
    Ok(parse::parse_branches(&stdout_str(&output)))
}

/// Checks out `branch`, creating it first when `create` is `true`.
pub fn checkout(cwd: &Path, branch: &str, create: bool) -> Result<()> {
    let branch = branch.trim();
    if branch.is_empty() {
        bail!("branch name cannot be empty");
    }
    let mut args = vec!["checkout"];
    if create {
        args.push("-b");
    }
    args.push(branch);
    run(cwd, &args, "git checkout failed")?;
    Ok(())
}

/// Creates `name` without switching to it, optionally from `from`.
pub fn branch_create(cwd: &Path, name: &str, from: Option<&str>) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("branch name cannot be empty");
    }
    let mut args = vec!["branch", name];
    if let Some(from) = from.map(str::trim).filter(|from| !from.is_empty()) {
        args.push(from);
    }
    run(cwd, &args, "git branch failed")?;
    Ok(())
}

/// Deletes a local branch, using `-D` when `force` is `true`.
pub fn branch_delete(cwd: &Path, name: &str, force: bool) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("branch name cannot be empty");
    }
    let flag = if force { "-D" } else { "-d" };
    run(cwd, &["branch", flag, name], "git branch delete failed")?;
    Ok(())
}

/// Merges `branch` into the current branch and returns the command's stdout.
pub fn merge(cwd: &Path, branch: &str) -> Result<String> {
    let branch = branch.trim();
    if branch.is_empty() {
        bail!("branch name cannot be empty");
    }
    let output = run(cwd, &["merge", branch], "git merge failed")?;
    Ok(stdout_str(&output))
}

/// Resets the current HEAD to `target` (default `HEAD`), optionally hard.
pub fn reset(cwd: &Path, target: Option<&str>, hard: bool) -> Result<()> {
    let mut args = vec!["reset"];
    if hard {
        args.push("--hard");
    }
    if let Some(target) = target.map(str::trim).filter(|target| !target.is_empty()) {
        args.push(target);
    }
    run(cwd, &args, "git reset failed")?;
    Ok(())
}

/// Lists the stash entries, newest first.
pub fn stash_list(cwd: &Path) -> Result<Vec<GitStashEntry>> {
    let format = format!("--format={}", parse::STASH_FORMAT);
    let output = run(
        cwd,
        &["stash", "list", "-z", &format],
        "git stash list failed",
    )?;
    Ok(parse::parse_stashes(&stdout_str(&output)))
}

/// Stashes the current changes with an optional message.
pub fn stash_push(cwd: &Path, message: Option<&str>) -> Result<()> {
    let mut args = vec!["stash", "push"];
    if let Some(message) = message.map(str::trim).filter(|message| !message.is_empty()) {
        args.push("-m");
        args.push(message);
    }
    run(cwd, &args, "git stash failed")?;
    Ok(())
}

/// Pops the stash at `index` (default: the most recent entry).
pub fn stash_pop(cwd: &Path, index: Option<u32>) -> Result<()> {
    let selector = index.map(|index| format!("stash@{{{index}}}"));
    let mut args = vec!["stash", "pop"];
    if let Some(selector) = selector.as_deref() {
        args.push(selector);
    }
    run(cwd, &args, "git stash pop failed")?;
    Ok(())
}

/// Drops the stash at `index` (default: the most recent entry).
pub fn stash_drop(cwd: &Path, index: Option<u32>) -> Result<()> {
    let selector = index.map(|index| format!("stash@{{{index}}}"));
    let mut args = vec!["stash", "drop"];
    if let Some(selector) = selector.as_deref() {
        args.push(selector);
    }
    run(cwd, &args, "git stash drop failed")?;
    Ok(())
}

/// Lists tags. Only annotated tags expose a message.
pub fn tags(cwd: &Path) -> Result<Vec<GitTag>> {
    let format = format!("--format={}", parse::TAG_FORMAT);
    let output = run(cwd, &["tag", &format], "git tag failed")?;
    Ok(parse::parse_tags(&stdout_str(&output)))
}

/// Creates a tag, optionally annotated with `message` and pointing at `target`.
pub fn tag_create(
    cwd: &Path,
    name: &str,
    message: Option<&str>,
    target: Option<&str>,
) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("tag name cannot be empty");
    }
    let mut args = vec!["tag"];
    if let Some(message) = message.map(str::trim).filter(|message| !message.is_empty()) {
        args.push("-a");
        args.push(name);
        args.push("-m");
        args.push(message);
    } else {
        args.push(name);
    }
    if let Some(target) = target.map(str::trim).filter(|target| !target.is_empty()) {
        args.push(target);
    }
    run(cwd, &args, "git tag failed")?;
    Ok(())
}

/// Deletes a local tag.
pub fn tag_delete(cwd: &Path, name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("tag name cannot be empty");
    }
    run(cwd, &["tag", "-d", name], "git tag delete failed")?;
    Ok(())
}

/// Lists configured remotes with their fetch and push URLs.
pub fn remotes(cwd: &Path) -> Result<Vec<GitRemote>> {
    let output = run(cwd, &["remote", "-v"], "git remote failed")?;
    Ok(parse::parse_remotes(&stdout_str(&output)))
}

/// Adds a remote.
pub fn remote_add(cwd: &Path, name: &str, url: &str) -> Result<()> {
    let name = name.trim();
    let url = url.trim();
    if name.is_empty() {
        bail!("remote name cannot be empty");
    }
    if url.is_empty() {
        bail!("remote URL cannot be empty");
    }
    run(cwd, &["remote", "add", name, url], "git remote add failed")?;
    Ok(())
}

/// Removes a remote.
pub fn remote_remove(cwd: &Path, name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("remote name cannot be empty");
    }
    run(cwd, &["remote", "remove", name], "git remote remove failed")?;
    Ok(())
}

/// Renames a remote.
pub fn remote_rename(cwd: &Path, old: &str, new: &str) -> Result<()> {
    let old = old.trim();
    let new = new.trim();
    if old.is_empty() || new.is_empty() {
        bail!("remote name cannot be empty");
    }
    run(
        cwd,
        &["remote", "rename", old, new],
        "git remote rename failed",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init_repo() -> Option<tempfile::TempDir> {
        if !crate::cli::helpers::is_git_available() {
            eprintln!("skipping git tests: git is not available");
            return None;
        }
        let dir = tempfile::tempdir().expect("tempdir");
        let cwd = dir.path();
        git(cwd, &["init", "-q"]);
        git(cwd, &["config", "user.email", "gitnapse@test.local"]);
        git(cwd, &["config", "user.name", "GitNapse Test"]);
        git(cwd, &["config", "commit.gpgsign", "false"]);
        Some(dir)
    }

    fn git(cwd: &Path, args: &[&str]) {
        let output = crate::cli::helpers::run_git_with_cwd(args, cwd).expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            crate::cli::helpers::stderr_msg(&output)
        );
    }

    fn write_file(cwd: &Path, name: &str, contents: &str) {
        fs::write(cwd.join(name), contents).expect("write file");
    }

    fn seed_commit(cwd: &Path, name: &str, contents: &str, message: &str) {
        write_file(cwd, name, contents);
        git(cwd, &["add", "--", name]);
        git(cwd, &["commit", "-qm", message]);
    }

    #[test]
    fn status_reports_clean_staged_unstaged_and_untracked() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "tracked.txt", "one\n", "initial");

        let clean = status(cwd).expect("status");
        assert!(clean.clean);
        assert!(clean.branch.is_some());
        assert!(!clean.detached);
        assert!(clean.staged.is_empty());
        assert!(clean.unstaged.is_empty());

        write_file(cwd, "new.txt", "two\n");
        let untracked = status(cwd).expect("status");
        assert_eq!(untracked.untracked, vec!["new.txt".to_string()]);
        assert!(!untracked.clean);

        stage(cwd, &["new.txt".to_string()]).expect("stage");
        let staged = status(cwd).expect("status");
        assert!(staged.untracked.is_empty());
        assert_eq!(staged.staged.len(), 1);
        assert_eq!(staged.staged[0].path, "new.txt");
        assert_eq!(staged.staged[0].kind, "added");
        assert!(staged.staged[0].staged);

        commit(cwd, "add new", false).expect("commit");
        write_file(cwd, "tracked.txt", "changed\n");
        let unstaged = status(cwd).expect("status");
        assert_eq!(unstaged.unstaged.len(), 1);
        assert_eq!(unstaged.unstaged[0].path, "tracked.txt");
        assert_eq!(unstaged.unstaged[0].kind, "modified");
        assert!(!unstaged.unstaged[0].staged);
        assert!(!unstaged.clean);
    }

    #[test]
    fn unstage_and_discard_restore_index_and_worktree() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "tracked.txt", "one\n", "initial");

        write_file(cwd, "new.txt", "two\n");
        stage(cwd, &["new.txt".to_string()]).expect("stage");
        unstage(cwd, &["new.txt".to_string()]).expect("unstage");
        let current = status(cwd).expect("status");
        assert!(current.staged.is_empty());
        assert_eq!(current.untracked, vec!["new.txt".to_string()]);

        write_file(cwd, "tracked.txt", "dirty\n");
        discard(cwd, &["tracked.txt".to_string()]).expect("discard");
        assert_eq!(
            fs::read_to_string(cwd.join("tracked.txt")).expect("read file"),
            "one\n"
        );
        let after_discard = status(cwd).expect("status");
        assert!(after_discard.unstaged.is_empty());
        assert_eq!(after_discard.untracked, vec!["new.txt".to_string()]);
    }

    #[test]
    fn log_returns_parsed_history() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "a.txt", "one\n", "first commit");
        seed_commit(cwd, "b.txt", "two\n", "second commit");

        let entries = log(cwd, 10).expect("log");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].subject, "second commit");
        assert_eq!(entries[0].author_email, "gitnapse@test.local");
        assert!(!entries[0].hash.is_empty());
        assert!(entries[0].short.len() < entries[0].hash.len());

        assert!(log(cwd, 0).expect("log zero").is_empty());
    }

    #[test]
    fn branches_tags_and_stash_roundtrip() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "a.txt", "one\n", "initial");

        branch_create(cwd, "feature", None).expect("branch create");
        let branches_before = branches(cwd).expect("branches");
        assert!(
            branches_before
                .iter()
                .any(|branch| branch.name == "feature" && !branch.current)
        );

        checkout(cwd, "feature", false).expect("checkout");
        let branches_after = branches(cwd).expect("branches");
        assert!(
            branches_after
                .iter()
                .any(|branch| branch.name == "feature" && branch.current)
        );

        branch_create(cwd, "topic", None).expect("topic branch");
        branch_delete(cwd, "topic", false).expect("delete topic");
        assert!(
            !branches(cwd)
                .expect("branches")
                .iter()
                .any(|branch| branch.name == "topic")
        );

        tag_create(cwd, "v1", Some("release one"), None).expect("tag create");
        let tag_list = tags(cwd).expect("tags");
        let v1 = tag_list
            .iter()
            .find(|tag| tag.name == "v1")
            .expect("v1 tag");
        assert_eq!(v1.message.as_deref(), Some("release one"));
        assert!(v1.date.is_some());
        tag_delete(cwd, "v1").expect("tag delete");
        assert!(tags(cwd).expect("tags").is_empty());

        write_file(cwd, "a.txt", "stashed\n");
        stash_push(cwd, Some("wip")).expect("stash push");
        let stashes = stash_list(cwd).expect("stash list");
        assert_eq!(stashes.len(), 1);
        assert_eq!(stashes[0].index, 0);
        assert_eq!(stashes[0].name, "stash@{0}");
        assert!(stashes[0].message.contains("wip"));
        assert!(status(cwd).expect("status").unstaged.is_empty());

        stash_pop(cwd, None).expect("stash pop");
        assert!(stash_list(cwd).expect("stash list").is_empty());
        assert_eq!(status(cwd).expect("status").unstaged.len(), 1);

        stash_push(cwd, None).expect("stash push again");
        stash_drop(cwd, Some(0)).expect("stash drop");
        assert!(stash_list(cwd).expect("stash list").is_empty());
    }

    #[test]
    fn diff_covers_worktree_staged_commit_and_range() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "a.txt", "one\n", "initial");
        write_file(cwd, "a.txt", "two\n");

        let worktree = diff(cwd, &DiffMode::Worktree { path: None }).expect("worktree diff");
        assert!(worktree.contains("-one"));
        assert!(worktree.contains("+two"));

        stage(cwd, &["a.txt".to_string()]).expect("stage");
        let staged = diff(cwd, &DiffMode::Staged { path: None }).expect("staged diff");
        assert!(staged.contains("+two"));

        let hash = commit(cwd, "update a", false).expect("commit");
        let commit_diff = diff(
            cwd,
            &DiffMode::Commit {
                rev: format!("{hash}~1"),
                path: None,
            },
        )
        .expect("commit diff");
        assert!(commit_diff.contains("+two"));

        let range = diff(
            cwd,
            &DiffMode::Range {
                from: "HEAD~1".to_string(),
                to: "HEAD".to_string(),
                path: None,
            },
        )
        .expect("range diff");
        assert!(range.contains("+two"));
    }

    #[test]
    fn merge_and_reset_update_the_worktree() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "a.txt", "one\n", "initial");
        let main = status(cwd).expect("status").branch.expect("current branch");

        branch_create(cwd, "topic", None).expect("branch create");
        checkout(cwd, "topic", false).expect("checkout topic");
        seed_commit(cwd, "b.txt", "topic\n", "topic commit");
        checkout(cwd, &main, false).expect("checkout main");

        let output = merge(cwd, "topic").expect("merge");
        assert!(output.contains("Fast-forward") || output.contains("Merge made"));
        assert!(cwd.join("b.txt").exists());

        reset(cwd, Some("HEAD~1"), true).expect("reset");
        assert!(!cwd.join("b.txt").exists());
    }

    #[test]
    fn repo_info_and_remote_management() {
        let Some(dir) = init_repo() else {
            return;
        };
        let cwd = dir.path();
        seed_commit(cwd, "a.txt", "one\n", "initial");

        remote_add(cwd, "origin", "https://github.com/owner/repo.git").expect("remote add");
        let remote_list = remotes(cwd).expect("remotes");
        assert_eq!(remote_list.len(), 1);
        assert_eq!(remote_list[0].name, "origin");
        assert_eq!(
            remote_list[0].fetch_url,
            "https://github.com/owner/repo.git"
        );

        let info = repo_info(cwd).expect("repo info");
        assert_eq!(info.full_name.as_deref(), Some("owner/repo"));
        assert_eq!(
            info.remote.as_deref(),
            Some("https://github.com/owner/repo.git")
        );
        assert!(info.branch.is_some());
        assert!(!info.detached);
        assert_eq!(
            info.name,
            cwd.file_name()
                .expect("dir name")
                .to_string_lossy()
                .to_string()
        );

        remote_rename(cwd, "origin", "upstream").expect("remote rename");
        assert_eq!(remotes(cwd).expect("remotes")[0].name, "upstream");
        remote_remove(cwd, "upstream").expect("remote remove");
        assert!(remotes(cwd).expect("remotes").is_empty());
    }
}
