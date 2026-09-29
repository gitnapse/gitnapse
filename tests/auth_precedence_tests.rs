use gitnapse::auth;
use serial_test::serial;

#[test]
#[serial]
fn env_token_has_precedence_over_stored_sources() {
    temp_env::with_var("GITHUB_TOKEN", Some("env-priority-token"), || {
        let loaded = auth::load_token().expect("load token");
        assert_eq!(loaded.as_deref(), Some("env-priority-token"));
    });
}

#[test]
#[serial]
fn token_source_matches_env_precedence_without_returning_secret() {
    temp_env::with_var("GITHUB_TOKEN", Some("env-priority-token"), || {
        let source = auth::token_source().expect("token source");
        assert_eq!(source, auth::TokenSource::Env);
        assert!(source.has_token());
        assert_eq!(source.label(), "GITHUB_TOKEN env");
    });
}
