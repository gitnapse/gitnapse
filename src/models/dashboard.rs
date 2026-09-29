use serde::{Deserialize, Serialize};

use super::IssueUser;

/// Public profile information for a GitHub user.
///
/// Returned by `GET /users/{login}` and by user search (`GET /search/users`).
/// Search results are reduced user objects, so numeric fields fall back to `0`
/// when GitHub omits them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub login: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blog: Option<String>,
    #[serde(default)]
    pub followers: u64,
    #[serde(default)]
    pub following: u64,
    #[serde(default)]
    pub public_repos: u64,
    pub html_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// A comment on an issue or on a pull request conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: u64,
    pub user: IssueUser,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
}

/// A single code search hit ("repo" is the repository `full_name`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub repo: String,
    pub path: String,
    pub name: String,
    pub sha: String,
    pub html_url: String,
}

/// A compact, display-oriented public activity event.
///
/// `kind` is one of `push`, `pull_request`, `issues`, `release`, `create`,
/// `watch`, `fork` or `other`; `title` is a short human summary when the
/// event payload is understood, otherwise `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub id: String,
    pub kind: String,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_avatar_url: Option<String>,
    pub repo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub created_at: String,
}

/// A thread from the authenticated user's GitHub notifications inbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub unread: bool,
    pub reason: String,
    pub subject_type: String,
    pub subject_title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
}

/// Bytes of code per language for a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStat {
    pub name: String,
    pub bytes: u64,
}

/// A repository contributor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub login: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub contributions: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
}
