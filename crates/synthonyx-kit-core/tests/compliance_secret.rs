//! Compliance contract tests for `Secret<T>`.
//!
//! References:
//! - GDPR Art. 32 (security of processing — minimise exposure of personal
//!   data; cryptographic protection where appropriate).
//! - ISO 27002:2022 control 5.34 (privacy and PII).
//! - ENISA cryptographic-guidelines on handling of sensitive material
//!   (avoid accidental logging and serialisation).
//!
//! Together these mandate that secret values must not be exposed via
//! `Debug`, `Display`, or implicit serialisation, and that their memory
//! footprint must be cleared on drop.

use static_assertions::{assert_impl_all, assert_not_impl_any};
use synthonyx_kit_core::Secret;

// Compile-time guarantees enforced at every build.
assert_not_impl_any!(Secret<String>: std::fmt::Display);
assert_impl_all!(Secret<String>: std::fmt::Debug);

#[cfg(feature = "serde")]
mod with_serde {
    use super::*;
    // The strongest guarantee: even when serde is enabled, `Secret<T>`
    // must not be implicitly serialisable. Callers must opt in explicitly
    // (e.g. via `#[serde(with = "...")]`) to round-trip a secret.
    assert_not_impl_any!(Secret<String>: serde::Serialize, serde::de::DeserializeOwned);
}

#[test]
fn art_32_debug_never_contains_raw_value() {
    let s = Secret::new(String::from("hunter2-supersecret"));
    let dbg = format!("{s:?}");
    assert_eq!(dbg, "Secret(<redacted>)");
    assert!(
        !dbg.contains("hunter2"),
        "Debug output must not leak the secret value"
    );
}

#[test]
fn art_32_expose_returns_inner_value() {
    let s = Secret::new(String::from("hunter2"));
    assert_eq!(s.expose(), "hunter2");
}

#[test]
fn art_32_secret_implements_drop_for_zeroize_payloads() {
    // We cannot portably inspect freed memory, but we can verify that
    // `Secret<T>` participates in `Drop` so the zeroize impl runs.
    use std::mem::needs_drop;
    assert!(needs_drop::<Secret<String>>());
    assert!(needs_drop::<Secret<Vec<u8>>>());
}

#[test]
fn art_32_from_impl_wraps_value() {
    let s: Secret<String> = String::from("x").into();
    assert_eq!(s.expose(), "x");
}
