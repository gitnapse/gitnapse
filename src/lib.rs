pub mod app;
pub mod auth;
pub mod cache;
pub mod cli;
pub mod config;
pub mod error;
pub mod github;
pub mod models;
pub mod provider;
pub mod registry;
pub mod runtime;
pub mod syntax;
pub mod task_manager;

// Auth-related modules live under `src/auth/`; re-exported here to keep the
// public API (`gitnapse::oauth`, etc.) stable.
pub use auth::{oauth, oauth_session, secure_store};
