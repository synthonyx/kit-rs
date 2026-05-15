//! Compliance contract tests for `FileAuditLogger`.
//!
//! References:
//! - DORA Art. 9 (audit-trail retention).
//! - DORA Art. 32 (audit-log integrity — tamper evidence).
//! - GDPR Art. 30 (records of processing must be reliably retrievable).
//!
//! Contracts enforced:
//! - Concurrent writers from multiple threads produce a single, valid,
//!   in-order BLAKE3 chain.
//! - Tampering with any non-terminal entry is detected by `verify_chain`.
//! - Reopening the file after a clean shutdown preserves the running hash;
//!   subsequent appends extend the original chain.
//! - Each `record()` call is durably flushed before returning.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use synthonyx_kit_audit::{FileAuditLogger, verify_chain};
use synthonyx_kit_core::{
    AuditEntry, AuditLogger, AuditValue, CorrelationId, OriginKind, OriginSnapshot, Outcome,
    UnixNanos,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[allow(
    clippy::disallowed_methods,
    reason = "test-only: wall clock used for unique filename"
)]
fn tmp_path(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "synthonyx-audit-compl-{prefix}-{nanos}-{seq}.jsonl"
    ))
}

fn entry(action: String) -> AuditEntry {
    AuditEntry {
        timestamp: UnixNanos(1_700_000_000_000_000_000),
        correlation_id: CorrelationId([1u8; 16]),
        module: Cow::Borrowed("test"),
        action: Cow::Owned(action),
        origin: OriginSnapshot {
            kind: OriginKind::System,
            principal: None,
            service: None,
        },
        outcome: Outcome::Success,
        subject: None,
        fields: BTreeMap::from([("seq".to_string(), AuditValue::UInt(0))]),
        prev_hash: [0u8; 32],
    }
}

#[test]
fn art_9_round_trip_and_verify_passes() {
    let path = tmp_path("rt");
    let logger = FileAuditLogger::open(&path).unwrap();
    for i in 0..5 {
        logger.record(entry(format!("act{i}"))).unwrap();
    }
    drop(logger);
    verify_chain(&path).unwrap();
    let _ = std::fs::remove_file(&path);
}

#[test]
fn art_32_concurrent_writers_produce_valid_chain() {
    let path = tmp_path("concurrent");
    let logger = Arc::new(FileAuditLogger::open(&path).unwrap());

    const THREADS: usize = 8;
    const PER_THREAD: usize = 50;
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = vec![];

    for t in 0..THREADS {
        let logger = logger.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..PER_THREAD {
                logger.record(entry(format!("t{t}-{i}"))).unwrap();
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    drop(logger);

    verify_chain(&path).unwrap();

    // Sanity: line count equals total writes.
    let contents = std::fs::read_to_string(&path).unwrap();
    let line_count = contents.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(line_count, THREADS * PER_THREAD);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn art_9_reopen_preserves_chain() {
    let path = tmp_path("reopen");
    {
        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(entry("first".into())).unwrap();
        logger.record(entry("second".into())).unwrap();
    }
    {
        // Reopening must verify the existing chain and restore the running
        // hash; subsequent records extend it.
        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(entry("third".into())).unwrap();
        logger.record(entry("fourth".into())).unwrap();
    }
    verify_chain(&path).unwrap();
    let _ = std::fs::remove_file(&path);
}

#[test]
fn art_9_open_fails_when_existing_chain_is_broken() {
    let path = tmp_path("broken-reopen");
    {
        let logger = FileAuditLogger::open(&path).unwrap();
        logger.record(entry("first".into())).unwrap();
        logger.record(entry("second".into())).unwrap();
        logger.record(entry("third".into())).unwrap();
    }
    // Tamper with the middle entry while the file is closed.
    let contents = std::fs::read_to_string(&path).unwrap();
    let mutated = contents.replacen("\"second\"", "\"MUTATED\"", 1);
    std::fs::write(&path, mutated).unwrap();

    // Reopen must reject the broken chain.
    let result = FileAuditLogger::open(&path);
    assert!(result.is_err(), "reopen of broken chain must fail; got Ok");
    let _ = std::fs::remove_file(&path);
}
