//! Compliance contract tests for `OriginTrait` and `BaseOrigin<P>`.
//!
//! References:
//! - GDPR Art. 30 (records of processing must attribute every operation to
//!   a controller / processor — represented here by `OriginTrait::principal`
//!   and `kind`).
//! - DORA Art. 17 (incident attribution requires a stable correlation id
//!   carried by every dispatch).

use synthonyx_kit_core::{BaseOrigin, CorrelationId, OriginKind, OriginTrait};

fn cid(byte: u8) -> CorrelationId {
    CorrelationId([byte; 16])
}

fn variants() -> Vec<(BaseOrigin<String>, CorrelationId, OriginKind)> {
    vec![
        (
            BaseOrigin::System {
                correlation: cid(1),
            },
            cid(1),
            OriginKind::System,
        ),
        (
            BaseOrigin::Service {
                name: "svc-a",
                principal: "spiffe://prod/svc-a".to_string(),
                correlation: cid(2),
            },
            cid(2),
            OriginKind::Service,
        ),
        (
            BaseOrigin::User {
                principal: "user-42".to_string(),
                correlation: cid(3),
            },
            cid(3),
            OriginKind::User,
        ),
        (
            BaseOrigin::Anonymous {
                correlation: cid(4),
            },
            cid(4),
            OriginKind::Anonymous,
        ),
    ]
}

#[test]
fn art_30_every_variant_carries_correlation_id() {
    for (origin, expected_cid, _) in variants() {
        assert_eq!(origin.correlation_id(), expected_cid);
    }
}

#[test]
fn art_17_kind_matches_variant() {
    for (origin, _, expected_kind) in variants() {
        assert_eq!(origin.kind(), expected_kind);
    }
}

#[test]
fn art_30_principal_present_only_for_service_and_user() {
    for (origin, _, kind) in variants() {
        match kind {
            OriginKind::System | OriginKind::Anonymous => {
                assert!(
                    origin.principal().is_none(),
                    "{kind:?} must have no principal"
                );
            }
            OriginKind::Service | OriginKind::User => {
                assert!(
                    origin.principal().is_some(),
                    "{kind:?} must have a principal"
                );
            }
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn origin_kind_serde_roundtrips() {
    for variant in [
        OriginKind::System,
        OriginKind::Service,
        OriginKind::User,
        OriginKind::Anonymous,
    ] {
        let s = serde_json::to_string(&variant).unwrap();
        let round: OriginKind = serde_json::from_str(&s).unwrap();
        assert_eq!(round, variant);
    }
}
