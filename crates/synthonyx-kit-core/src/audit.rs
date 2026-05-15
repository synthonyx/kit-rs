//! Audit log trait, entry types, and outcomes.
//!
//! The `AuditLogger` trait lives in `-core` rather than `-audit` so
//! [`crate::Config::Audit`] can bound on it without a circular dependency.
//! Concrete sinks (`FileAuditLogger`, `TracingAuditLogger`) ship in
//! `synthonyx-kit-audit`.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::correlation::CorrelationId;
use crate::origin::OriginKind;
use crate::time::UnixNanos;

/// Outcome of an audited dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Outcome {
    /// Operation completed successfully.
    Success,
    /// Operation was denied (authorization, validation, etc.).
    Denied,
    /// Operation failed due to an error.
    Error,
}

/// Serializable, low-cardinality snapshot of an origin for audit storage.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OriginSnapshot {
    /// Origin kind.
    pub kind: OriginKind,
    /// Stable principal identifier as a short string (account id, SPIFFE id, etc.).
    pub principal: Option<String>,
    /// Service name (for [`OriginKind::Service`]). Borrowed when built from
    /// `Config::MODULE` / a `&'static str` literal; owned when deserialized.
    pub service: Option<Cow<'static, str>>,
}

/// Safe-to-log value type for audit entry fields.
///
/// Deliberately restricted to non-PII primitive shapes. To log fields derived
/// from PII, wrap the value in [`crate::Pii`] at the call site and store a
/// hash or token here instead of the raw value.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum AuditValue {
    /// A short, non-PII string (status code, kind tag, etc.).
    Str(String),
    /// A signed integer.
    Int(i64),
    /// An unsigned integer.
    UInt(u64),
    /// A boolean flag.
    Bool(bool),
    /// A hex-encoded hash (e.g. SHA256 of a sensitive identifier).
    Hash(String),
}

/// A single audit log entry.
///
/// Entries are appended in order. `prev_hash` carries the BLAKE3 hash of the
/// previous entry's canonical encoding, forming a tamper-evident chain. The
/// first entry's `prev_hash` is all zeros.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AuditEntry {
    /// Wall-clock timestamp from `Config::Time`.
    pub timestamp: UnixNanos,
    /// Correlation id of the originating operation.
    pub correlation_id: CorrelationId,
    /// Module identifier (from `Config::MODULE`). Borrowed at construction,
    /// owned on deserialization.
    pub module: Cow<'static, str>,
    /// Action name (e.g. `create_user`, `verify_password`). Borrowed at
    /// construction, owned on deserialization.
    pub action: Cow<'static, str>,
    /// Snapshot of the dispatch origin.
    pub origin: OriginSnapshot,
    /// Outcome of the operation.
    pub outcome: Outcome,
    /// Optional GDPR data subject linkage (opaque, pseudonymous identifier).
    pub subject: Option<String>,
    /// Structured, safe-to-log fields. `BTreeMap` so JSON encoding is
    /// canonical (sorted keys).
    pub fields: BTreeMap<String, AuditValue>,
    /// BLAKE3-32 hash of the previous entry's canonical encoding.
    pub prev_hash: [u8; 32],
}

/// Errors produced by an [`AuditLogger`] implementation.
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    /// I/O error writing or reading the audit log.
    #[error("audit I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Serialization error encoding or decoding an entry.
    #[error("audit serialization error: {0}")]
    Serialization(String),
    /// The audit log chain is broken (tamper indication).
    #[error("audit chain broken: expected prev_hash {expected}, got {actual}")]
    ChainBroken {
        /// Expected previous hash, hex-encoded.
        expected: String,
        /// Actual previous hash on the entry being appended, hex-encoded.
        actual: String,
    },
}

/// Sink for audit entries.
///
/// Implementations must:
/// - persist or transmit each entry durably enough for the deployment's
///   evidence-retention requirements (DORA Article 9);
/// - never silently drop entries;
/// - return [`AuditError`] rather than panicking on backpressure.
pub trait AuditLogger: Send + Sync + 'static {
    /// Append `entry` to the log.
    fn record(&self, entry: AuditEntry) -> Result<(), AuditError>;
}
