//! `Pii<T, C>` — type-level tagging of personal data.

use core::fmt;
use core::marker::PhantomData;

/// A type-level tag identifying a category of personal data.
///
/// Storage backends and audit sinks dispatch on `CATEGORY` to drive
/// column-level encryption, erasure handlers, and log redaction.
pub trait PiiCategory: Send + Sync + 'static {
    /// A stable, lower-case identifier (e.g. `"personal"`, `"gdpr-art-9"`).
    const CATEGORY: &'static str;
}

/// Standard personal data (GDPR Art. 4(1)).
#[derive(Clone, Copy, Debug, Default)]
pub struct Personal;
impl PiiCategory for Personal {
    const CATEGORY: &'static str = "personal";
}

/// Special category data (GDPR Art. 9 — racial origin, health, biometric, etc.).
#[derive(Clone, Copy, Debug, Default)]
pub struct Sensitive;
impl PiiCategory for Sensitive {
    const CATEGORY: &'static str = "gdpr-art-9";
}

/// Pseudonymous identifier (GDPR Art. 4(5)).
#[derive(Clone, Copy, Debug, Default)]
pub struct Pseudonymous;
impl PiiCategory for Pseudonymous {
    const CATEGORY: &'static str = "pseudonymous";
}

/// A value of type `T` tagged with PII category `C`.
///
/// `Debug` redacts the value; serde (when enabled) round-trips it
/// transparently so storage backends can apply category-aware handling.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Pii<T, C: PiiCategory>(
    T,
    #[cfg_attr(feature = "serde", serde(skip))] PhantomData<C>,
);

impl<T, C: PiiCategory> Pii<T, C> {
    /// Tag `value` as PII of category `C`.
    pub const fn new(value: T) -> Self {
        Self(value, PhantomData)
    }

    /// Unwrap to the inner value. Verbose by design.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Borrow the inner value.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// The category tag carried by this `Pii<T, C>`.
    pub const fn category() -> &'static str {
        C::CATEGORY
    }
}

impl<T: Clone, C: PiiCategory> Clone for Pii<T, C> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: fmt::Debug, C: PiiCategory> fmt::Debug for Pii<T, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pii<{}>(<redacted>)", C::CATEGORY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_value_and_shows_category() {
        let v = Pii::<_, Sensitive>::new(String::from("123-45-6789"));
        let debug = format!("{v:?}");
        assert_eq!(debug, "Pii<gdpr-art-9>(<redacted>)");
        assert!(!debug.contains("123-45-6789"));
    }

    #[test]
    fn category_constants() {
        assert_eq!(Personal::CATEGORY, "personal");
        assert_eq!(Sensitive::CATEGORY, "gdpr-art-9");
        assert_eq!(Pseudonymous::CATEGORY, "pseudonymous");
    }
}
