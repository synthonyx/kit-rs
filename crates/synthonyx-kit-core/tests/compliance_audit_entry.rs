//! Compliance contract tests for `AuditEntry` shape and serialisation.
//!
//! References:
//! - DORA Art. 9 (audit trail retention — entries must be persistable
//!   and re-readable verbatim).
//! - DORA Art. 32 (audit log integrity — entries must be deterministically
//!   encoded so a hash chain is reproducible).
//! - GDPR Art. 30 (records of processing).

use std::borrow::Cow;
use std::collections::BTreeMap;

use synthonyx_kit_core::{
    AuditEntry, AuditValue, CorrelationId, OriginKind, OriginSnapshot, Outcome, UnixNanos,
};

fn sample() -> AuditEntry {
    AuditEntry {
        timestamp: UnixNanos(1_700_000_000_000_000_000),
        correlation_id: CorrelationId([1u8; 16]),
        module: Cow::Borrowed("test"),
        action: Cow::Borrowed("act"),
        origin: OriginSnapshot {
            kind: OriginKind::System,
            principal: None,
            service: None,
        },
        outcome: Outcome::Success,
        subject: None,
        fields: BTreeMap::from([
            ("z".to_string(), AuditValue::Bool(true)),
            ("a".to_string(), AuditValue::UInt(1)),
        ]),
        prev_hash: [0u8; 32],
    }
}

#[test]
fn art_30_prev_hash_starts_at_zero_for_first_entry() {
    let e = sample();
    assert_eq!(e.prev_hash, [0u8; 32]);
}

#[cfg(feature = "serde")]
#[test]
fn art_32_serialises_deterministically() {
    let s1 = serde_json::to_string(&sample()).unwrap();
    let s2 = serde_json::to_string(&sample()).unwrap();
    assert_eq!(
        s1, s2,
        "AuditEntry serialisation must be byte-stable for chain hashing"
    );
}

#[cfg(feature = "serde")]
#[test]
fn art_32_fields_encoded_in_btreemap_order() {
    let s = serde_json::to_string(&sample()).unwrap();
    let a_pos = s.find("\"a\":").expect("`a` key present");
    let z_pos = s.find("\"z\":").expect("`z` key present");
    assert!(
        a_pos < z_pos,
        "BTreeMap-backed `fields` must encode in sorted-key order"
    );
}

#[cfg(feature = "serde")]
#[test]
fn art_9_roundtrip_via_serde_preserves_core_fields() {
    let e = sample();
    let s = serde_json::to_string(&e).unwrap();
    let round: AuditEntry = serde_json::from_str(&s).unwrap();
    assert_eq!(round.action, e.action);
    assert_eq!(round.module, e.module);
    assert_eq!(round.timestamp, e.timestamp);
    assert_eq!(round.correlation_id, e.correlation_id);
    assert_eq!(round.prev_hash, e.prev_hash);
    assert_eq!(round.outcome, e.outcome);
}

#[cfg(feature = "serde")]
#[test]
fn audit_value_untagged_serialises_compactly() {
    let json = serde_json::to_string(&AuditValue::Int(-7)).unwrap();
    assert_eq!(json, "-7");
    let json = serde_json::to_string(&AuditValue::Bool(true)).unwrap();
    assert_eq!(json, "true");
    let json = serde_json::to_string(&AuditValue::Str("ok".to_string())).unwrap();
    assert_eq!(json, "\"ok\"");
}
