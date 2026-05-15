//! Backend-agnostic storage abstraction for the Synthonyx Kit.
//!
//! Phase 1 defines the trait surface only:
//! - [`Codec`]: serde-based value encoding contract.
//! - [`Backend`]: raw byte-level KV interface.
//! - [`TransactionalBackend`]: marker for backends that support transactions.
//! - [`EncryptedBackend`], [`RetentionBackend`], [`ErasureBackend`]:
//!   compliance hooks that Phase 2 backends (Postgres, etc.) implement.
//! - [`StorageValue`], [`StorageMap`], [`StorageDoubleMap`]: typed storage
//!   declarations RTMs use to express their persistent shape.
//!
//! Concrete backends ship as separate crates in Phase 2
//! (`synthonyx-kit-storage-memory`, `-postgres`, `-sqlite`, ...).
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

use thiserror::Error;

/// A canonical, opaque storage key.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StorageKey(pub Vec<u8>);

impl StorageKey {
    /// Construct a `StorageKey` from a byte sequence.
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }
    /// Borrow the underlying bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Identifier of an encryption key, used for rotation tracking and audit.
#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct KeyId(pub String);

/// Encoding/decoding error.
#[derive(Debug, Error)]
pub enum CodecError {
    /// Encode failed.
    #[error("encode failed: {0}")]
    Encode(String),
    /// Decode failed.
    #[error("decode failed: {0}")]
    Decode(String),
}

/// Errors produced by storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Underlying I/O / driver error.
    #[error("storage I/O error: {0}")]
    Io(String),
    /// Codec round-trip failed.
    #[error("storage codec error: {0}")]
    Codec(#[from] CodecError),
    /// Transaction was aborted.
    #[error("transaction aborted: {0}")]
    TransactionAborted(String),
    /// Backend-specific error.
    #[error("storage backend error: {0}")]
    Backend(String),
    /// Erasure operation failed.
    #[error("erasure error: {0}")]
    Erasure(String),
}

/// Encoding/decoding contract for values stored in a [`Backend`].
///
/// Implementors choose their wire format (binary serde, protobuf, etc.).
/// Phase 2 backends typically supply a blanket impl for
/// `T: Serialize + DeserializeOwned` using their chosen codec (default:
/// `bincode2`); types that need a specific format (e.g. protobuf for gRPC
/// payloads) override `Codec` directly.
pub trait Codec: Sized + Send + Sync + 'static {
    /// Encode `self` to bytes.
    fn encode(&self) -> Result<Vec<u8>, CodecError>;
    /// Decode `Self` from bytes.
    fn decode(bytes: &[u8]) -> Result<Self, CodecError>;
}

/// Raw byte-level storage backend.
#[trait_variant::make(BackendSend: Send)]
pub trait Backend: Send + Sync + 'static {
    /// Read the value at `key`, or `None` if absent.
    async fn read(&self, key: &StorageKey) -> Result<Option<Vec<u8>>, StorageError>;
    /// Write `value` to `key`, overwriting any existing value.
    async fn write(&self, key: &StorageKey, value: &[u8]) -> Result<(), StorageError>;
    /// Delete the value at `key`. Idempotent.
    async fn delete(&self, key: &StorageKey) -> Result<(), StorageError>;
    /// Return whether `key` is present without reading the value.
    async fn exists(&self, key: &StorageKey) -> Result<bool, StorageError>;
}

/// Backend that supports atomic multi-key transactions.
///
/// Phase 1 declares the marker only; concrete transactional APIs ship in
/// Phase 2 alongside the first backend that supports them.
pub trait TransactionalBackend: Backend {}

/// Backend that encrypts data at rest.
pub trait EncryptedBackend: Backend {
    /// Whether this backend encrypts data at rest (column-level or TDE).
    fn encrypts_at_rest(&self) -> bool;
    /// Identifier of the currently active encryption key.
    fn key_id(&self) -> KeyId;
}

/// Backend that supports applying a retention policy.
pub trait RetentionBackend: Backend {
    /// Apply `policy`: delete entries older than its `max_age`. Returns the
    /// number of entries purged.
    fn apply_retention(
        &self,
        policy: &RetentionPolicy,
    ) -> impl core::future::Future<Output = Result<u64, StorageError>> + Send;
}

/// A retention policy for stored data (GDPR Art. 5(1)(e)).
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct RetentionPolicy {
    /// Maximum age of an entry, in nanoseconds, before it must be purged.
    pub max_age_nanos: u128,
}

/// Backend that supports erasing all entries linked to a data subject
/// (GDPR Art. 17, right to erasure).
pub trait ErasureBackend: Backend {
    /// Erase all entries referencing `subject_id`. Returns a structured
    /// report so callers can verify and log the operation.
    fn erase_subject(
        &self,
        subject_id: &str,
    ) -> impl core::future::Future<Output = Result<ErasureReport, StorageError>> + Send;
}

/// Report returned by [`ErasureBackend::erase_subject`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ErasureReport {
    /// Number of entries erased.
    pub entries_erased: u64,
    /// Optional list of storage keys that were affected (omitted by
    /// backends where listing is expensive).
    pub keys: Option<Vec<StorageKey>>,
}

/// A typed single-value storage slot.
pub trait StorageValue: 'static {
    /// The value type stored at this slot.
    type Value: Codec;
    /// The storage key.
    fn key() -> StorageKey;
}

/// A typed key→value map.
pub trait StorageMap: 'static {
    /// The map key type.
    type Key: Codec;
    /// The value type.
    type Value: Codec;
    /// The shared prefix all entries in this map share.
    fn prefix() -> StorageKey;
    /// Compute the full storage key for `key` by appending its encoded form
    /// to the map's prefix.
    fn full_key(key: &Self::Key) -> Result<StorageKey, CodecError> {
        let encoded = key.encode()?;
        let mut full = Self::prefix().0;
        full.extend_from_slice(&encoded);
        Ok(StorageKey(full))
    }
}

/// A typed (key1, key2) → value map.
pub trait StorageDoubleMap: 'static {
    /// First-level key.
    type Key1: Codec;
    /// Second-level key.
    type Key2: Codec;
    /// Value type.
    type Value: Codec;
    /// Shared prefix.
    fn prefix() -> StorageKey;
}
