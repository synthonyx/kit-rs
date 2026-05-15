//! Property-based tests for the BLAKE3-chained audit log.
//!
//! Two properties:
//! - For any sequence of 1..20 entries, `verify_chain` succeeds.
//! - For any sequence of 3..15 entries, mutating one byte inside a
//!   *non-terminal* entry causes `verify_chain` to fail with either
//!   `ChainBroken` (the chain detected the divergence) or `Serialization`
//!   (the mutation broke JSON parsing). The terminal entry is excluded
//!   because its mutation is undetectable without a successor — a
//!   documented limit of any forward-only hash chain.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use proptest::prelude::*;
use synthonyx_kit_audit::{FileAuditLogger, verify_chain};
use synthonyx_kit_core::{
    AuditEntry, AuditLogger, AuditValue, CorrelationId, OriginKind, OriginSnapshot, Outcome,
    UnixNanos,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_path(prefix: &str) -> PathBuf {
    let pid = std::process::id();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("synthonyx-audit-prop-{prefix}-{pid}-{seq}.jsonl"))
}

fn entry_for(action: String) -> AuditEntry {
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

proptest! {
    #[test]
    fn chain_verifies_for_any_length(actions in prop::collection::vec("[a-z]{1,8}", 1..20usize)) {
        let path = tmp_path("verify");
        let logger = FileAuditLogger::open(&path).unwrap();
        for action in &actions {
            logger.record(entry_for(action.clone())).unwrap();
        }
        drop(logger);
        verify_chain(&path).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tamper_in_non_terminal_entry_is_detected(
        n in 3..15usize,
        position_seed in any::<u32>(),
        bit_flip in 0u8..8u8,
    ) {
        let path = tmp_path("tamper");
        let logger = FileAuditLogger::open(&path).unwrap();
        for i in 0..n {
            logger.record(entry_for(format!("act{i}"))).unwrap();
        }
        drop(logger);

        let bytes = std::fs::read(&path).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();

        // Find the start of the last line (after the second-to-last newline).
        // All bytes before that index are in non-terminal entries.
        let last_nl = text.rfind('\n').expect("at least one newline");
        let non_terminal_end = text[..last_nl].rfind('\n').map_or(0, |i| i + 1);
        // We need at least one byte in [0, non_terminal_end) to mutate.
        prop_assume!(non_terminal_end > 0);

        let target = (position_seed as usize) % non_terminal_end;
        let mut mutated = bytes.clone();
        mutated[target] ^= 1u8 << bit_flip;
        prop_assume!(mutated != bytes);
        std::fs::write(&path, &mutated).unwrap();

        // Any error variant is a valid detection: ChainBroken (chain hash
        // mismatch), Serialization (JSON parse failed), or Io (UTF-8 broken
        // when reading lines). Compliance only requires that the file can
        // no longer be read back as the original log.
        match verify_chain(&path) {
            Err(_) => {}
            Ok(_) => panic!(
                "tamper at byte {target} (bit {bit_flip}) was NOT detected\nfile: {}",
                path.display()
            ),
        }
        let _ = std::fs::remove_file(&path);
    }
}
