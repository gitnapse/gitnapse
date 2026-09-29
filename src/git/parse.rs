use std::collections::HashMap;

use super::{FileChange, GitBranch, GitLogEntry, GitRemote, GitStashEntry, GitStatus, GitTag};

pub(crate) const LOG_FORMAT: &str = "%H%x00%h%x00%an%x00%ae%x00%aI%x00%s";
pub(crate) const BRANCH_FORMAT: &str =
    "%(refname:short)%00%(HEAD)%00%(upstream:short)%00%(upstream:track)";
pub(crate) const TAG_FORMAT: &str =
    "%(refname:short)%00%(objecttype)%00%(subject)%00%(creatordate:iso-strict)";
pub(crate) const STASH_FORMAT: &str = "%gd%x00%gs%x00%cI";
pub(crate) const LOG_FIELDS: usize = 6;

pub(crate) fn parse_status(raw: &str) -> GitStatus {
    let mut status = GitStatus {
        branch: None,
        detached: false,
        ahead: 0,
        behind: 0,
        clean: true,
        staged: Vec::new(),
        unstaged: Vec::new(),
        untracked: Vec::new(),
        conflicted: Vec::new(),
    };

    let mut records = raw.split('\0');
    while let Some(record) = records.next() {
        if record.is_empty() {
            continue;
        }

        if let Some(head) = record.strip_prefix("# branch.head ") {
            if head == "(detached)" {
                status.detached = true;
            } else {
                status.branch = Some(head.to_string());
            }
        } else if let Some(ahead_behind) = record.strip_prefix("# branch.ab ") {
            for token in ahead_behind.split_whitespace() {
                if let Some(ahead) = token.strip_prefix('+') {
                    status.ahead = ahead.parse().unwrap_or(0);
                } else if let Some(behind) = token.strip_prefix('-') {
                    status.behind = behind.parse().unwrap_or(0);
                }
            }
        } else if let Some(rest) = record.strip_prefix("1 ") {
            parse_changed_entry(rest, 8, None, &mut status);
        } else if let Some(rest) = record.strip_prefix("2 ") {
            let orig_path = records.next().map(str::to_string);
            parse_changed_entry(rest, 9, orig_path, &mut status);
        } else if let Some(rest) = record.strip_prefix("u ") {
            if let Some(path) = last_field(rest, 10) {
                status.conflicted.push(path.to_string());
            }
        } else if let Some(path) = record.strip_prefix("? ") {
            status.untracked.push(path.to_string());
        }
    }

    status.clean = status.staged.is_empty()
        && status.unstaged.is_empty()
        && status.untracked.is_empty()
        && status.conflicted.is_empty();
    status
}

fn parse_changed_entry(
    rest: &str,
    fields: usize,
    orig_path: Option<String>,
    status: &mut GitStatus,
) {
    let Some((xy, path)) = split_xy_path(rest, fields) else {
        return;
    };
    let mut codes = xy.chars();
    let Some(staged_code) = codes.next() else {
        return;
    };
    let unstaged_code = codes.next().unwrap_or('.');

    if let Some(kind) = change_kind(staged_code) {
        status.staged.push(FileChange {
            path: path.to_string(),
            orig_path: orig_path.clone(),
            kind,
            staged: true,
        });
    }
    if let Some(kind) = change_kind(unstaged_code) {
        status.unstaged.push(FileChange {
            path: path.to_string(),
            orig_path,
            kind,
            staged: false,
        });
    }
}

fn split_xy_path(rest: &str, fields: usize) -> Option<(&str, &str)> {
    let tokens: Vec<&str> = rest.splitn(fields, ' ').collect();
    if tokens.len() < fields {
        return None;
    }
    Some((tokens[0], tokens[fields - 1]))
}

fn last_field(rest: &str, fields: usize) -> Option<&str> {
    let tokens: Vec<&str> = rest.splitn(fields, ' ').collect();
    if tokens.len() < fields {
        return None;
    }
    tokens.last().copied()
}

fn change_kind(code: char) -> Option<String> {
    let kind = match code {
        'M' => "modified",
        'A' => "added",
        'D' => "deleted",
        'R' => "renamed",
        'C' => "copied",
        'T' => "typechange",
        'U' => "unmerged",
        _ => return None,
    };
    Some(kind.to_string())
}

pub(crate) fn parse_log(raw: &str) -> Vec<GitLogEntry> {
    raw.split('\0')
        .collect::<Vec<_>>()
        .chunks(LOG_FIELDS)
        .filter_map(|chunk| {
            if chunk.len() < LOG_FIELDS || chunk[0].is_empty() {
                return None;
            }
            Some(GitLogEntry {
                hash: chunk[0].to_string(),
                short: chunk[1].to_string(),
                author_name: chunk[2].to_string(),
                author_email: chunk[3].to_string(),
                date: chunk[4].to_string(),
                subject: chunk[5].to_string(),
            })
        })
        .collect()
}

pub(crate) fn parse_branches(raw: &str) -> Vec<GitBranch> {
    raw.lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }
            let fields: Vec<&str> = line.split('\0').collect();
            let name = fields.first().copied().unwrap_or_default().trim();
            if name.is_empty() {
                return None;
            }
            let current = fields.get(1).is_some_and(|head| head.trim() == "*");
            let upstream = fields
                .get(2)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let (ahead, behind) = parse_track(fields.get(3).copied().unwrap_or_default());
            Some(GitBranch {
                name: name.to_string(),
                current,
                upstream,
                ahead,
                behind,
            })
        })
        .collect()
}

fn parse_track(track: &str) -> (u32, u32) {
    let mut ahead = 0;
    let mut behind = 0;
    for part in track.trim_matches(|ch| ch == '[' || ch == ']').split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("ahead ") {
            ahead = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = part.strip_prefix("behind ") {
            behind = value.trim().parse().unwrap_or(0);
        }
    }
    (ahead, behind)
}

pub(crate) fn parse_tags(raw: &str) -> Vec<GitTag> {
    raw.lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }
            let fields: Vec<&str> = line.split('\0').collect();
            let name = fields.first().copied().unwrap_or_default().trim();
            if name.is_empty() {
                return None;
            }
            let object_type = fields.get(1).copied().unwrap_or_default().trim();
            let subject = fields.get(2).copied().unwrap_or_default().trim();
            let date = fields
                .get(3)
                .copied()
                .unwrap_or_default()
                .trim()
                .to_string();
            let message = if object_type == "tag" && !subject.is_empty() {
                Some(subject.to_string())
            } else {
                None
            };
            Some(GitTag {
                name: name.to_string(),
                message,
                date: (!date.is_empty()).then_some(date),
            })
        })
        .collect()
}

pub(crate) fn parse_stashes(raw: &str) -> Vec<GitStashEntry> {
    raw.split('\0')
        .collect::<Vec<_>>()
        .chunks(3)
        .filter_map(|chunk| {
            if chunk.len() < 3 || chunk[0].is_empty() {
                return None;
            }
            let selector = chunk[0].trim();
            let index = selector
                .strip_prefix("stash@{")
                .and_then(|value| value.strip_suffix('}'))
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            Some(GitStashEntry {
                index,
                name: selector.to_string(),
                message: chunk[1].to_string(),
                date: chunk[2].trim().to_string(),
            })
        })
        .collect()
}

pub(crate) fn parse_remotes(raw: &str) -> Vec<GitRemote> {
    let mut names: Vec<String> = Vec::new();
    let mut fetch_by_name: HashMap<String, String> = HashMap::new();
    let mut push_by_name: HashMap<String, String> = HashMap::new();

    for line in raw.lines() {
        let Some((name, rest)) = line.split_once('\t') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let (url, is_push) = if let Some(url) = rest.strip_suffix(" (push)") {
            (url.trim(), true)
        } else if let Some(url) = rest.strip_suffix(" (fetch)") {
            (url.trim(), false)
        } else {
            (rest.trim(), false)
        };
        if !fetch_by_name.contains_key(name) && !push_by_name.contains_key(name) {
            names.push(name.to_string());
        }
        if is_push {
            push_by_name.insert(name.to_string(), url.to_string());
        } else {
            fetch_by_name.insert(name.to_string(), url.to_string());
        }
    }

    names
        .into_iter()
        .filter_map(|name| {
            let fetch_url = fetch_by_name.remove(&name)?;
            let push_url = push_by_name.remove(&name).filter(|push| *push != fetch_url);
            Some(GitRemote {
                name,
                fetch_url,
                push_url,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_parses_clean_branch() {
        let raw = "# branch.oid abc\0# branch.head main\0# branch.ab +2 -3\0";
        let status = parse_status(raw);
        assert_eq!(status.branch.as_deref(), Some("main"));
        assert!(!status.detached);
        assert_eq!(status.ahead, 2);
        assert_eq!(status.behind, 3);
        assert!(status.clean);
    }

    #[test]
    fn status_parses_detached_head() {
        let status = parse_status("# branch.oid abc\0# branch.head (detached)\0");
        assert!(status.detached);
        assert!(status.branch.is_none());
    }

    #[test]
    fn status_parses_staged_unstaged_untracked_and_conflicts() {
        let raw = concat!(
            "# branch.head main\0",
            "1 A. N... 100644 100644 100644 a b staged.txt\0",
            "1 .M N... 100644 100644 100644 a b changed file.txt\0",
            "2 R. N... 100644 100644 100644 a b R100 renamed.txt\0old name.txt\0",
            "u UU N... 100644 100644 100644 100644 a b c conflict.txt\0",
            "? untracked.txt\0",
        );
        let status = parse_status(raw);

        assert_eq!(status.staged.len(), 2);
        assert_eq!(status.staged[0].path, "staged.txt");
        assert_eq!(status.staged[0].kind, "added");
        assert!(status.staged[0].staged);
        assert_eq!(status.staged[1].kind, "renamed");
        assert_eq!(status.staged[1].orig_path.as_deref(), Some("old name.txt"));

        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.unstaged[0].path, "changed file.txt");
        assert_eq!(status.unstaged[0].kind, "modified");
        assert!(!status.unstaged[0].staged);

        assert_eq!(status.untracked, vec!["untracked.txt"]);
        assert_eq!(status.conflicted, vec!["conflict.txt"]);
        assert!(!status.clean);
    }

    #[test]
    fn log_parses_nul_separated_entries() {
        let raw = concat!(
            "hash1\0short1\0Alice\0alice@example.com\u{0}2026-01-01T00:00:00+00:00\0subject one\0",
            "hash2\0short2\0Bob\0bob@example.com\u{0}2026-01-02T00:00:00+00:00\0subject two\0",
        );
        let entries = parse_log(raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "hash1");
        assert_eq!(entries[0].subject, "subject one");
        assert_eq!(entries[1].author_name, "Bob");
    }

    #[test]
    fn branches_parse_upstream_tracking() {
        let raw = "main\0*\0origin/main\0[ahead 1, behind 2]\nfeature\0 \0\0\ngone\0 \0origin/gone\0[gone]\n";
        let branches = parse_branches(raw);
        assert_eq!(branches.len(), 3);
        assert!(branches[0].current);
        assert_eq!(branches[0].upstream.as_deref(), Some("origin/main"));
        assert_eq!((branches[0].ahead, branches[0].behind), (1, 2));
        assert!(!branches[1].current);
        assert_eq!((branches[2].ahead, branches[2].behind), (0, 0));
    }

    #[test]
    fn tags_parse_annotated_and_lightweight() {
        let raw = "v1\0tag\0release one\u{0}2026-01-01T00:00:00+00:00\nlight\0commit\0commit subject\u{0}2026-01-02T00:00:00+00:00\n";
        let tags = parse_tags(raw);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].message.as_deref(), Some("release one"));
        assert_eq!(tags[1].message, None);
        assert!(tags[1].date.is_some());
    }

    #[test]
    fn stashes_parse_selector_message_and_date() {
        let raw = "stash@{0}\0On main: wip\u{0}2026-01-01T00:00:00+00:00\0stash@{1}\0WIP on main: abc\u{0}2026-01-02T00:00:00+00:00\0";
        let stashes = parse_stashes(raw);
        assert_eq!(stashes.len(), 2);
        assert_eq!(stashes[0].index, 0);
        assert_eq!(stashes[0].name, "stash@{0}");
        assert!(stashes[0].message.contains("wip"));
        assert_eq!(stashes[1].index, 1);
    }

    #[test]
    fn remotes_merge_fetch_and_push_urls() {
        let raw = "origin\thttps://github.com/owner/repo.git (fetch)\norigin\thttps://github.com/owner/repo.git (push)\nupstream\thttps://example.com/up.git (fetch)\nupstream\tgit@example.com:up.git (push)\n";
        let remotes = parse_remotes(raw);
        assert_eq!(remotes.len(), 2);
        assert_eq!(remotes[0].name, "origin");
        assert_eq!(remotes[0].fetch_url, "https://github.com/owner/repo.git");
        assert_eq!(remotes[0].push_url, None);
        assert_eq!(
            remotes[1].push_url.as_deref(),
            Some("git@example.com:up.git")
        );
    }
}
