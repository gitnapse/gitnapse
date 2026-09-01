use crate::error::AuthError;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

const KEYRING_SERVICE: &str = "com.GitNapse.GitNapse";

/// The storage backend used for a saved secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretBackend {
    /// Secret is stored in the operating system's keyring.
    Keyring,
    /// Secret is stored in a local file with restricted permissions.
    File,
}

fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok_and(|v| !v.trim().is_empty()) {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(version) = fs::read_to_string("/proc/version")
            && version.to_ascii_lowercase().contains("microsoft")
        {
            return true;
        }
    }
    false
}

fn should_try_keyring() -> bool {
    !is_wsl()
}

fn ensure_keyring_init() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        let _ = keyring::use_native_store(false);
    });
}

fn keyring_get(secret_key: &str) -> Option<Result<Option<String>, AuthError>> {
    if !should_try_keyring() {
        return None;
    }
    ensure_keyring_init();
    let entry = keyring_core::Entry::new(KEYRING_SERVICE, secret_key);
    match entry {
        Ok(entry) => match entry.get_password() {
            Ok(value) => Some(Ok(Some(value))),
            Err(err) => {
                log::warn!(
                    "keyring get_password failed for secret '{}' (service '{}'): {}. Falling back to file storage.",
                    secret_key,
                    KEYRING_SERVICE,
                    err
                );
                Some(Ok(None))
            }
        },
        Err(error) => Some(Err(AuthError::Keyring(error.to_string()))),
    }
}

fn keyring_set(secret_key: &str, value: &str) -> Option<Result<(), AuthError>> {
    if !should_try_keyring() {
        return None;
    }
    ensure_keyring_init();
    let entry = keyring_core::Entry::new(KEYRING_SERVICE, secret_key);
    match entry {
        Ok(entry) => {
            let result = entry
                .set_password(value)
                .map_err(|e| AuthError::Keyring(e.to_string()));
            if let Err(ref err) = result {
                log::warn!(
                    "keyring set_password failed for secret '{}' (service '{}'): {}. Falling back to file storage.",
                    secret_key,
                    KEYRING_SERVICE,
                    err
                );
            }
            Some(result)
        }
        Err(error) => Some(Err(AuthError::Keyring(error.to_string()))),
    }
}

fn keyring_delete(secret_key: &str) -> Option<Result<(), AuthError>> {
    if !should_try_keyring() {
        return None;
    }
    ensure_keyring_init();
    let entry = keyring_core::Entry::new(KEYRING_SERVICE, secret_key);
    match entry {
        Ok(entry) => Some(
            entry
                .delete_credential()
                .map_err(|e| AuthError::Keyring(e.to_string())),
        ),
        Err(error) => Some(Err(AuthError::Keyring(error.to_string()))),
    }
}

fn file_read(path: &Path) -> Result<Option<String>, AuthError> {
    if !path.exists() {
        return Ok(None);
    }
    let value = fs::read_to_string(path).map_err(|e| {
        AuthError::Other(format!("Cannot read secret file '{}': {e}", path.display()))
    })?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed))
}

fn file_write(path: &Path, value: &str) -> Result<(), AuthError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AuthError::Other(format!(
                "Cannot create secret directory '{}': {e}",
                parent.display()
            ))
        })?;
    }
    fs::write(path, format!("{value}\n")).map_err(|e| {
        AuthError::Other(format!(
            "Cannot write secret file '{}': {e}",
            path.display()
        ))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            AuthError::Other(format!(
                "Cannot set permissions on '{}': {e}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

fn file_delete(path: &Path) -> Result<(), AuthError> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| {
            AuthError::Other(format!(
                "Cannot remove secret file '{}': {e}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

/// Saves a secret to the operating system's keyring, falling back to a local
/// file if the keyring is unavailable (e.g., on WSL or headless systems).
///
/// If the keyring save succeeds, any existing fallback file is removed to avoid
/// stale credentials. Returns the [`SecretBackend`] that was used.
///
/// # Errors
/// Returns an error if both the keyring and the fallback file write fail.
pub fn save_secret(
    secret_key: &str,
    fallback_file: &Path,
    value: &str,
) -> Result<SecretBackend, AuthError> {
    if let Some(result) = keyring_set(secret_key, value)
        && result.is_ok()
    {
        let _ = file_delete(fallback_file);
        return Ok(SecretBackend::Keyring);
    }
    file_write(fallback_file, value)?;
    Ok(SecretBackend::File)
}

/// Loads a secret from the operating system's keyring, falling back to a local
/// file if the keyring does not contain the secret.
///
/// Returns `Ok(None)` if the secret does not exist in either backend.
///
/// # Errors
/// Returns an error if reading from either backend fails unexpectedly.
pub fn load_secret(secret_key: &str, fallback_file: &Path) -> Result<Option<String>, AuthError> {
    if let Some(result) = keyring_get(secret_key)
        && let Ok(Some(value)) = result
    {
        return Ok(Some(value));
    }
    file_read(fallback_file)
}

/// Removes a secret from both the operating system's keyring and the local
/// fallback file.
///
/// # Errors
/// Returns an error if deleting the fallback file fails. Keyring deletion
/// errors are logged as warnings but do not cause the function to fail.
pub fn clear_secret(secret_key: &str, fallback_file: &Path) -> Result<(), AuthError> {
    if let Some(Err(e)) = keyring_delete(secret_key) {
        log::warn!(
            "failed to clear keyring secret '{}' (service '{}'): {}. Continuing with file cleanup.",
            secret_key,
            KEYRING_SERVICE,
            e
        );
    }
    file_delete(fallback_file)
}

/// Returns a human-readable name of the preferred secret storage backend
/// (`"keyring"` or `"file-fallback"`) based on the current environment.
pub fn preferred_backend_name() -> &'static str {
    if should_try_keyring() {
        "keyring"
    } else {
        "file-fallback"
    }
}
