# synthonyx-kit-compliance

EU regulatory type taxonomy for the Synthonyx Kit. The shared vocabulary
that downstream RTMs, storage backends, and (Phase 2) the incident-reporting
crate dispatch on.

## Contents

- `DataSubjectId` — opaque pseudonymous identifier under GDPR Art. 4(5).
  Should never be derived directly from raw PII.
- `GdprCategory` — `Personal` / `Sensitive` / `Pseudonymous` / `Anonymous`.
- `IncidentClass` — `Minor` / `Significant` / `Major` /
  `SevereCyberThreat`, ordered by severity. DORA Art. 18 classification.
- `Nis2Class` — `Essential` / `Important` (NIS2 Art. 3).
- `Erasable` — marker trait for RTMs that store data linked to a
  `DataSubjectId`. Phase 2's runtime composer fans out
  `Erasable::erase(subject)` across all matching RTMs for GDPR Art. 17.
- Re-exports of [`Severity`](../synthonyx-kit-core) and
  [`RetentionPolicy`](../synthonyx-kit-storage) for ergonomic use.

These types are intentionally small — they form a wire-level contract
when persisted in audit entries and incident reports. Variant ordering and
serde encoding are nailed down by
[`compliance_classification.rs`](tests/compliance_classification.rs).
