//! `Secret<T>` — guarded wrapper for sensitive values.

use core::fmt;
use zeroize::Zeroize;

/// A wrapper that protects sensitive values from accidental disclosure.
///
/// `Secret<T>`:
/// - redacts in `Debug` output (`Secret(<redacted>)`);
/// - zeroizes its contents on `Drop`;
/// - intentionally does **not** implement `Serialize`/`Deserialize` even when
///   the `serde` feature is enabled — to round-trip a secret, callers must
///   use `#[serde(with = "...")]` or call [`Self::expose`] explicitly.
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    /// Wrap `value` as a secret.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Borrow the underlying secret value.
    ///
    /// The verbose name is deliberate: every call site is a place a reviewer
    /// should examine.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Mutably borrow the underlying secret value.
    pub fn expose_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl<T: Zeroize> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts() {
        let s = Secret::new(String::from("hunter2"));
        let debug = format!("{s:?}");
        assert_eq!(debug, "Secret(<redacted>)");
        assert!(!debug.contains("hunter2"));
    }

    #[test]
    fn expose_returns_inner() {
        let s = Secret::new(String::from("hunter2"));
        assert_eq!(s.expose(), "hunter2");
    }
}
