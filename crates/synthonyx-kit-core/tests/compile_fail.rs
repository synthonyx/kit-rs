//! trybuild driver for compile-fail compliance tests.
//!
//! The cases under `tests/compile-fail/` prove at compile time that
//! `Secret<T>` cannot be implicitly serialised and `Pii<T, C>` cannot be
//! interpolated via `Display`.
//!
//! Run `TRYBUILD=overwrite cargo test --all-features -p synthonyx-kit-core
//! compile_fails` once after Rust toolchain bumps to refresh the `.stderr`
//! snapshots; otherwise the test verifies the existing snapshots.

#[cfg(feature = "serde")]
#[test]
fn compile_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
