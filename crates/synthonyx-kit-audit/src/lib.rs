//! Concrete audit log sinks for the Synthonyx Kit.
//!
//! The [`AuditLogger`] trait and [`AuditEntry`] type live in
//! [`synthonyx_kit_core`]. This crate provides reference sinks:
//!
//! - [`FileAuditLogger`]: append-only, BLAKE3-chained, tamper-evident.
//! - [`TracingAuditLogger`]: emits entries via the `tracing` crate for
//!   development and observability.
//! - [`verify_chain`]: standalone verifier for a stored audit log file.
//!
//! On every `record()`, [`FileAuditLogger`] overwrites the entry's
//! `prev_hash` field with the BLAKE3 hash of the previous entry's serialized
//! form (`[0u8; 32]` for the first entry). This means callers can leave
//! `prev_hash` zero when building entries — the logger fills it in.
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use synthonyx_kit_core::{AuditEntry, AuditError, AuditLogger};

// Re-export common audit types so consumers only need one crate import.
pub use synthonyx_kit_core::{AuditValue, OriginSnapshot, Outcome};

/// Append-only, BLAKE3-chained audit log writer.
///
/// Each entry is serialized to canonical JSON (one entry per line). The
/// `prev_hash` field on each entry equals BLAKE3 of the previous serialized
/// line (sans trailing newline). The first entry's `prev_hash` is all zeros.
///
/// Verify the file independently via [`verify_chain`].
pub struct FileAuditLogger {
    inner: Mutex<State>,
}

struct State {
    file: File,
    last_hash: [u8; 32],
    #[allow(dead_code)]
    path: PathBuf,
}

impl FileAuditLogger {
    /// Open or create an audit log file at `path`.
    ///
    /// If the file already exists, its chain is verified end-to-end and the
    /// running hash is restored from the last entry. Opening fails with
    /// [`AuditError::ChainBroken`] if the on-disk chain is broken.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AuditError> {
        let path = path.as_ref().to_path_buf();
        let last_hash = if path.exists() {
            verify_chain(&path)?
        } else {
            [0u8; 32]
        };
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            inner: Mutex::new(State {
                file,
                last_hash,
                path,
            }),
        })
    }
}

impl AuditLogger for FileAuditLogger {
    fn record(&self, mut entry: AuditEntry) -> Result<(), AuditError> {
        let mut state = self.inner.lock().expect("file audit logger mutex poisoned");
        entry.prev_hash = state.last_hash;
        let line =
            serde_json::to_string(&entry).map_err(|e| AuditError::Serialization(e.to_string()))?;
        let new_hash = *blake3::hash(line.as_bytes()).as_bytes();
        let mut bytes = line.into_bytes();
        bytes.push(b'\n');
        state.file.write_all(&bytes)?;
        state.file.flush()?;
        state.last_hash = new_hash;
        Ok(())
    }
}

/// Verify the chain of a stored audit log file end-to-end.
///
/// Returns the BLAKE3 hash of the final entry on success (this is the value
/// the next appended entry's `prev_hash` must equal).
///
/// Returns [`AuditError::ChainBroken`] on the first mismatch and
/// [`AuditError::Serialization`] on the first malformed line.
pub fn verify_chain(path: impl AsRef<Path>) -> Result<[u8; 32], AuditError> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut expected_prev = [0u8; 32];
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let entry: AuditEntry =
            serde_json::from_str(&line).map_err(|e| AuditError::Serialization(e.to_string()))?;
        if entry.prev_hash != expected_prev {
            return Err(AuditError::ChainBroken {
                expected: hex::encode(expected_prev),
                actual: hex::encode(entry.prev_hash),
            });
        }
        expected_prev = *blake3::hash(line.as_bytes()).as_bytes();
    }
    Ok(expected_prev)
}

/// Audit sink that emits entries via the `tracing` crate.
///
/// Intended for development and observability. Not a sufficient sole sink
/// for DORA-regulated production deployments; pair with a persistent sink
/// such as [`FileAuditLogger`] or a Phase 2 storage-backed logger.
pub struct TracingAuditLogger;

impl AuditLogger for TracingAuditLogger {
    fn record(&self, entry: AuditEntry) -> Result<(), AuditError> {
        let payload =
            serde_json::to_string(&entry).map_err(|e| AuditError::Serialization(e.to_string()))?;
        ::tracing::info!(
            target: "synthonyx::audit",
            module = %entry.module,
            action = %entry.action,
            outcome = ?entry.outcome,
            correlation_id = %entry.correlation_id,
            "{payload}",
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::BTreeMap;

    use synthonyx_kit_core::{
        AuditEntry, AuditValue, CorrelationId, OriginKind, OriginSnapshot, Outcome, UnixNanos,
    };

    use super::*;

    #[allow(
        clippy::disallowed_methods,
        reason = "test-only: wall clock used for unique filename suffix"
    )]
    fn tmp_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("synthonyx-audit-{name}-{nanos}.jsonl"))
    }

    fn sample_entry(action: &'static str) -> AuditEntry {
        AuditEntry {
            timestamp: UnixNanos(1_700_000_000_000_000_000),
            correlation_id: CorrelationId([1u8; 16]),
            module: Cow::Borrowed("test"),
            action: Cow::Borrowed(action),
            origin: OriginSnapshot {
                kind: OriginKind::System,
                principal: None,
                service: None,
            },
            outcome: Outcome::Success,
            subject: None,
            fields: BTreeMap::from([("count".to_string(), AuditValue::UInt(7))]),
            prev_hash: [0u8; 32],
        }
    }

    #[test]
    fn round_trip_record_and_verify() {
        let path = tmp_path("rt");
        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(sample_entry("first")).unwrap();
        logger.record(sample_entry("second")).unwrap();
        logger.record(sample_entry("third")).unwrap();
        drop(logger);

        verify_chain(&path).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reopen_restores_running_hash() {
        let path = tmp_path("reopen");
        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(sample_entry("first")).unwrap();
        drop(logger);

        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(sample_entry("second")).unwrap();
        drop(logger);

        verify_chain(&path).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tamper_in_middle_breaks_chain() {
        // Tampering with a *terminal* entry cannot be caught by chained
        // hashes alone (there is no successor to bind it). Record three
        // entries and mutate the middle one — the third entry's `prev_hash`
        // then disagrees with `BLAKE3(mutated_second)`.
        let path = tmp_path("tamper");
        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(sample_entry("first")).unwrap();
        logger.record(sample_entry("second")).unwrap();
        logger.record(sample_entry("third")).unwrap();
        drop(logger);

        let contents = std::fs::read_to_string(&path).unwrap();
        let mutated = contents.replacen("\"second\"", "\"MUTATED\"", 1);
        assert_ne!(contents, mutated);
        std::fs::write(&path, mutated).unwrap();

        let err = verify_chain(&path).unwrap_err();
        assert!(
            matches!(err, AuditError::ChainBroken { .. }),
            "expected ChainBroken, got {err:?}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tracing_logger_does_not_error() {
        let logger = TracingAuditLogger;
        logger.record(sample_entry("traced")).unwrap();
    }
}
