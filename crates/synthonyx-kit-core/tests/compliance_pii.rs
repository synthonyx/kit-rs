//! Compliance contract tests for `Pii<T, C>`.
//!
//! References:
//! - GDPR Art. 4(1) (personal data definition) → `Pii<_, Personal>`.
//! - GDPR Art. 4(5) (pseudonymisation) → `Pii<_, Pseudonymous>`.
//! - GDPR Art. 9 (special category data) → `Pii<_, Sensitive>`.
//!
//! Each category tag must be visible in `Debug` so log readers can identify
//! the category without exposing the value. Display is intentionally not
//! implemented to prevent accidental string interpolation.

use static_assertions::assert_not_impl_any;
use synthonyx_kit_core::{Personal, Pii, PiiCategory, Pseudonymous, Sensitive};

// `Pii<T, C>` must NEVER implement `Display`, regardless of `T` or `C`.
// (We assert against a concrete type because trait-object negative bounds
// require a concrete `Self`.)
assert_not_impl_any!(Pii<String, Personal>: std::fmt::Display);
assert_not_impl_any!(Pii<String, Sensitive>: std::fmt::Display);
assert_not_impl_any!(Pii<String, Pseudonymous>: std::fmt::Display);

#[test]
fn art_4_1_personal_category_redacts_in_debug() {
    let v = Pii::<_, Personal>::new(String::from("Jane Doe"));
    let dbg = format!("{v:?}");
    assert_eq!(dbg, "Pii<personal>(<redacted>)");
    assert!(!dbg.contains("Jane"));
}

#[test]
fn art_9_sensitive_category_redacts_in_debug() {
    let v = Pii::<_, Sensitive>::new(String::from("HIV+"));
    let dbg = format!("{v:?}");
    assert_eq!(dbg, "Pii<gdpr-art-9>(<redacted>)");
    assert!(!dbg.contains("HIV"));
}

#[test]
fn art_4_5_pseudonymous_category_redacts_in_debug() {
    let v = Pii::<_, Pseudonymous>::new(String::from("token-abc-123"));
    let dbg = format!("{v:?}");
    assert_eq!(dbg, "Pii<pseudonymous>(<redacted>)");
    assert!(!dbg.contains("token"));
}

#[test]
fn category_constants_are_stable_strings() {
    // Storage backends and audit sinks dispatch on these strings; they form
    // a wire-level contract once persisted.
    assert_eq!(Personal::CATEGORY, "personal");
    assert_eq!(Sensitive::CATEGORY, "gdpr-art-9");
    assert_eq!(Pseudonymous::CATEGORY, "pseudonymous");
}

#[cfg(feature = "serde")]
#[test]
fn art_4_1_pii_serde_is_transparent() {
    // Pii must round-trip transparently — the storage backend, not the type,
    // is responsible for encryption / erasure routing.
    let v = Pii::<_, Personal>::new(String::from("Jane Doe"));
    let json = serde_json::to_string(&v).unwrap();
    assert_eq!(json, "\"Jane Doe\"");
    let round: Pii<String, Personal> = serde_json::from_str(&json).unwrap();
    assert_eq!(round.expose(), "Jane Doe");
}
