use gitnapse::secure_store;
use serial_test::serial;
use tempfile::tempdir;

#[test]
#[serial]
fn file_fallback_save_load_clear_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("secret-token");
    let key = "test_secret_roundtrip";

    let prev = std::env::var("WSL_DISTRO_NAME").ok();
    unsafe { std::env::set_var("WSL_DISTRO_NAME", "Ubuntu") };

    let backend = secure_store::save_secret(key, &file, "abc123").expect("save");
    assert_eq!(backend, secure_store::SecretBackend::File);

    let loaded = secure_store::load_secret(key, &file).expect("load");
    assert_eq!(loaded.as_deref(), Some("abc123"));

    secure_store::clear_secret(key, &file).expect("clear");
    let loaded_after_clear = secure_store::load_secret(key, &file).expect("load after clear");
    assert_eq!(loaded_after_clear, None);

    if let Some(value) = prev {
        unsafe { std::env::set_var("WSL_DISTRO_NAME", value) };
    } else {
        unsafe { std::env::remove_var("WSL_DISTRO_NAME") };
    }
}

#[test]
#[serial]
fn has_secret_tracks_file_backend_presence() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("secret-presence");
    let key = "test_secret_presence";

    let prev = std::env::var("WSL_DISTRO_NAME").ok();
    unsafe { std::env::set_var("WSL_DISTRO_NAME", "Ubuntu") };

    assert!(!secure_store::has_secret(key, &file));
    secure_store::save_secret(key, &file, "present").expect("save");
    assert!(secure_store::has_secret(key, &file));
    secure_store::clear_secret(key, &file).expect("clear");
    assert!(!secure_store::has_secret(key, &file));

    if let Some(value) = prev {
        unsafe { std::env::set_var("WSL_DISTRO_NAME", value) };
    } else {
        unsafe { std::env::remove_var("WSL_DISTRO_NAME") };
    }
}

#[test]
#[serial]
#[cfg(unix)]
fn file_fallback_sets_secure_permissions_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("secret-permissions");
    let key = "test_secret_permissions";

    let prev = std::env::var("WSL_DISTRO_NAME").ok();
    unsafe { std::env::set_var("WSL_DISTRO_NAME", "Ubuntu") };

    let backend = secure_store::save_secret(key, &file, "perm-check").expect("save");
    assert_eq!(backend, secure_store::SecretBackend::File);

    let metadata = std::fs::metadata(&file).expect("metadata");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);

    if let Some(value) = prev {
        unsafe { std::env::set_var("WSL_DISTRO_NAME", value) };
    } else {
        unsafe { std::env::remove_var("WSL_DISTRO_NAME") };
    }
}
