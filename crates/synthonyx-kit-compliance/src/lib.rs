//! EU regulatory type taxonomy.
//!
//! Phase 1 ships the shared vocabulary (data subject identifiers, GDPR/DORA/
//! NIS2 classifications, the [`Erasable`] trait). Phase 2 wires these into
//! storage backends and the DORA incident-reporting crate.
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

use thiserror::Error;

/// Opaque, pseudonymous identifier of a GDPR data subject.
///
/// Implementations should never derive this directly from raw PII; prefer a
/// UUID, hash, or other indirect token so the identifier itself is not
/// personal data under GDPR Art. 4(1).
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataSubjectId(pub String);

impl DataSubjectId {
    /// Construct a `DataSubjectId` from a string-convertible value.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// GDPR data category classification (Art. 4 / Art. 9).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GdprCategory {
    /// Standard personal data (Art. 4(1)).
    Personal,
    /// Special category data (Art. 9 — racial origin, health, biometric, etc.).
    Sensitive,
    /// Pseudonymous identifier (Art. 4(5)).
    Pseudonymous,
    /// Anonymous, non-personal data.
    Anonymous,
}

/// DORA incident classification (Art. 18).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IncidentClass {
    /// Minor; not in scope for reporting.
    Minor,
    /// Significant; may require internal notification.
    Significant,
    /// Major; triggers DORA Art. 19 reporting (24-hour clock).
    Major,
    /// Severe cyber threat (Art. 19(2)).
    SevereCyberThreat,
}

/// NIS2 entity classification (Art. 3).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Nis2Class {
    /// Essential entity (Annex I).
    Essential,
    /// Important entity (Annex II).
    Important,
}

/// Errors produced by erasure operations.
#[derive(Debug, Error)]
pub enum ErasureError {
    /// Underlying erasure backend failed.
    #[error("erasure backend failure: {0}")]
    Backend(String),
    /// The requested subject was not found in any erasure-aware RTM.
    #[error("subject not found: {0:?}")]
    SubjectNotFound(DataSubjectId),
}

/// Marker trait: this RTM stores personal data linkable to a [`DataSubjectId`]
/// and is in scope for GDPR Art. 17 erasure.
///
/// Runtime composers (hand-written today, `compose_runtime!`-generated in
/// Phase 3) fan out erasure calls across every RTM implementing this trait.
pub trait Erasable: Send + Sync + 'static {
    /// Erase all data linked to `subject`. Returns the number of records
    /// affected.
    fn erase(
        &self,
        subject: &DataSubjectId,
    ) -> impl core::future::Future<Output = Result<u64, ErasureError>> + Send;
}

pub use synthonyx_kit_core::Severity;
pub use synthonyx_kit_storage::RetentionPolicy;
