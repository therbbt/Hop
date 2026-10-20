//! Secrets (passwords, key passphrases, RDP credentials) never touch the
//! config JSON. They live in the OS keychain (macOS Keychain, Windows
//! Credential Manager, Secret Service on Linux) and the config only stores
//! an opaque handle string pointing at one.

use thiserror::Error;
use uuid::Uuid;

const SERVICE: &str = "com.hop.app";
/// Where secrets were stored while Hop was called Portus (up to 0.1.6).
/// [`retrieve`] falls back to it and copies what it finds into [`SERVICE`].
const LEGACY_SERVICE: &str = "com.portus.app";

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("keychain backend error: {0}")]
    Backend(#[from] keyring::Error),
}

/// Generates a fresh opaque handle and stores `secret` under it. The
/// returned handle is what gets written into an `AuthMethod` variant on
/// [`crate::config::SavedSession::auth`].
pub fn store(secret: &str) -> Result<String, KeychainError> {
    let handle = Uuid::new_v4().to_string();
    let entry = keyring::Entry::new(SERVICE, &handle)?;
    entry.set_password(secret)?;
    Ok(handle)
}

pub fn retrieve(handle: &str) -> Result<String, KeychainError> {
    let entry = keyring::Entry::new(SERVICE, handle)?;
    match entry.get_password() {
        Err(keyring::Error::NoEntry) => {}
        found => return Ok(found?),
    }
    // Not under Hop's name: it may have been saved before the rename. Copy
    // it over on first use so later lookups find it directly. The legacy
    // entry is left alone, since an older Portus install may still use it.
    let secret = keyring::Entry::new(LEGACY_SERVICE, handle)?.get_password()?;
    let _ = entry.set_password(&secret);
    Ok(secret)
}

pub fn delete(handle: &str) -> Result<(), KeychainError> {
    let entry = keyring::Entry::new(SERVICE, handle)?;
    entry.delete_credential()?;
    Ok(())
}
