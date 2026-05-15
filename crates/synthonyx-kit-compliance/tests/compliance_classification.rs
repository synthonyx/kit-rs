//! Compliance contract tests for regulatory taxonomy enums.
//!
//! References:
//! - DORA Art. 18 (incident classification — `IncidentClass`).
//! - NIS2 Art. 3 (essential / important entity classification — `Nis2Class`).
//! - GDPR Arts. 4 / 9 (personal data classification — `GdprCategory`).
//!
//! These enums are persisted in incident reports and storage metadata.
//! Their serde encoding therefore forms a wire-level contract; this test
//! pins the encoding so future variants are added at the end of each enum
//! and never re-ordered.

use serde::{Deserialize, Serialize};
use synthonyx_kit_compliance::{DataSubjectId, GdprCategory, IncidentClass, Nis2Class};

fn roundtrip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(v: T) {
    let s = serde_json::to_string(&v).unwrap();
    let parsed: T = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, v);
}

#[test]
fn dora_art_18_incident_class_serde_roundtrips_all_variants() {
    for v in [
        IncidentClass::Minor,
        IncidentClass::Significant,
        IncidentClass::Major,
        IncidentClass::SevereCyberThreat,
    ] {
        roundtrip(v);
    }
}

#[test]
fn dora_art_18_incident_class_canonical_encoding() {
    assert_eq!(
        serde_json::to_string(&IncidentClass::Minor).unwrap(),
        "\"Minor\""
    );
    assert_eq!(
        serde_json::to_string(&IncidentClass::Significant).unwrap(),
        "\"Significant\""
    );
    assert_eq!(
        serde_json::to_string(&IncidentClass::Major).unwrap(),
        "\"Major\""
    );
    assert_eq!(
        serde_json::to_string(&IncidentClass::SevereCyberThreat).unwrap(),
        "\"SevereCyberThreat\""
    );
}

#[test]
fn nis2_art_3_class_serde_roundtrips() {
    roundtrip(Nis2Class::Essential);
    roundtrip(Nis2Class::Important);
}

#[test]
fn gdpr_art_4_category_serde_roundtrips() {
    for v in [
        GdprCategory::Personal,
        GdprCategory::Sensitive,
        GdprCategory::Pseudonymous,
        GdprCategory::Anonymous,
    ] {
        roundtrip(v);
    }
}

#[test]
fn gdpr_art_4_data_subject_id_is_opaque_string() {
    let id = DataSubjectId::new("user-abc-123");
    assert_eq!(id.as_str(), "user-abc-123");
    let s = serde_json::to_string(&id).unwrap();
    assert_eq!(s, "\"user-abc-123\"");
}

#[test]
fn ordering_of_incident_severity_matches_escalation() {
    // The variants must be listed in increasing severity so callers can use
    // `if incident as u8 >= IncidentClass::Major as u8` for escalation
    // decisions. This is a wire-level + ordering contract.
    let order = [
        IncidentClass::Minor,
        IncidentClass::Significant,
        IncidentClass::Major,
        IncidentClass::SevereCyberThreat,
    ];
    for window in order.windows(2) {
        assert!((window[0] as u8) < (window[1] as u8));
    }
}
