//! Compliance contract tests for `Argon2Password`.
//!
//! References:
//! - ENISA cryptographic guidelines (Argon2id v19 with library defaults is
//!   the recommended password-hashing function for new systems).
//! - GDPR Art. 32 (authentication credentials must be protected against
//!   unauthorised access — hashed at rest, never logged in plaintext).
//! - ISO 27002:2022 control 8.5 (secure authentication information).
//!
//! The contract enforced here:
//! - Correct passwords verify; incorrect passwords reject without panic.
//! - Malformed stored hashes yield a typed `Verification` error, never a
//!   panic.
//! - The PHC-encoded hash uses Argon2id v19.
//! - `Debug` output never contains the hash string.

use synthonyx_kit_core::Secret;
use synthonyx_kit_password::{Argon2Password, PasswordChecker, PasswordError};

fn secret(s: &str) -> Secret<String> {
    Secret::new(s.to_string())
}

#[test]
fn enisa_phc_format_uses_argon2id_v19() {
    let p = Argon2Password::new(secret("password123")).unwrap();
    let h = p.as_hash_str();
    assert!(h.starts_with("$argon2id$"), "expected argon2id prefix: {h}");
    assert!(h.contains("v=19"), "expected v=19 in PHC: {h}");
}

#[test]
fn art_32_correct_password_verifies() {
    let p = Argon2Password::new(secret("correct-horse-battery-staple")).unwrap();
    assert!(p.verify(secret("correct-horse-battery-staple")).unwrap());
}

#[test]
fn art_32_wrong_password_rejected_without_panic() {
    let p = Argon2Password::new(secret("password123")).unwrap();
    let result = p.verify(secret("not-the-password"));
    assert!(matches!(result, Ok(false)));
}

#[test]
fn art_32_malformed_hash_returns_verification_error_not_panic() {
    // `from_hash` accepts any string; verification must reject malformed
    // input as an error rather than panicking.
    let p = Argon2Password::from_hash("not-a-real-phc-hash");
    let result = p.verify(secret("anything"));
    assert!(
        matches!(result, Err(PasswordError::Hashing(_))),
        "expected Hashing error for malformed PHC, got {result:?}"
    );
}

#[test]
fn art_32_debug_never_contains_hash_bytes() {
    let p = Argon2Password::new(secret("password123")).unwrap();
    let dbg = format!("{p:?}");
    assert_eq!(dbg, "Argon2Password(<redacted>)");
    assert!(!dbg.contains("$argon2"));
    assert!(!dbg.contains("$v="));
}

#[test]
fn art_32_clone_yields_same_hash() {
    // Cloning shares the underlying Arc<str>; the hash is identical.
    let p = Argon2Password::new(secret("password123")).unwrap();
    let p2 = p.clone();
    assert_eq!(p.as_hash_str(), p2.as_hash_str());
}

#[cfg(feature = "serde")]
#[test]
fn art_32_serde_roundtrip_preserves_verification() {
    let p = Argon2Password::new(secret("password123")).unwrap();
    let s = serde_json::to_string(&p).unwrap();
    let p2: Argon2Password = serde_json::from_str(&s).unwrap();
    assert_eq!(p.as_hash_str(), p2.as_hash_str());
    assert!(p2.verify(secret("password123")).unwrap());
    assert!(!p2.verify(secret("wrong")).unwrap());
}
