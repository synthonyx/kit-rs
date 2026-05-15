# Regulatory compliance matrix

This document maps each regulatory requirement the Synthonyx Kit explicitly
addresses to the primitive(s) that implement it and the test file(s) that
prove the implementation upholds the contract.

The matrix is **load-bearing**: `cargo test -p compliance-matrix-test`
parses this file, asserts every referenced test path exists on disk, and
asserts every `crates/*/tests/compliance_*.rs` and
`crates/*/tests/proptest_*.rs` is referenced here. Any drift fails CI.

## Scope and limitations

- **EU first.** GDPR, DORA, NIS2, and eIDAS 2.0 are the priority frameworks.
  Other entries (ISO 27001, ENISA guidelines) appear where they reinforce
  the EU controls.
- **Kit-level scope only.** The kit provides the primitives; achieving
  end-to-end compliance is the responsibility of the service that composes
  RTMs on top. Storage encryption-at-rest, retention sweeps, incident
  reporting deadlines, and similar operational concerns are implemented in
  Phase 2 backends and are marked `Phase 2` below.
- **Evidence chain.** Each row links to a specific Rust test file whose
  assertions implement the requirement at the code level. Phase 2 will
  extend this with integration tests that exercise the same paths against
  real backends.

## GDPR — General Data Protection Regulation (EU 2016/679)

| Article | Requirement (summary) | Kit primitive | Test file |
|---|---|---|---|
| Art. 4(1) | Definition of personal data | `Pii<T, Personal>` | `crates/synthonyx-kit-core/tests/compliance_pii.rs` |
| Art. 4(5) | Pseudonymisation | `Pii<T, Pseudonymous>` | `crates/synthonyx-kit-core/tests/compliance_pii.rs` |
| Art. 5(1)(e) | Storage limitation — retention | `RetentionPolicy`, `RetentionBackend` (Phase 2) | `crates/synthonyx-kit-compliance/tests/compliance_classification.rs` |
| Art. 9 | Special-category personal data | `Pii<T, Sensitive>` | `crates/synthonyx-kit-core/tests/compliance_pii.rs` |
| Art. 9 | Audit-trail retrievability (re-serialisation) | `AuditEntry` | `crates/synthonyx-kit-core/tests/compliance_audit_entry.rs` |
| Art. 17 | Right to erasure | `Erasable`, `ErasureBackend` (Phase 2) | `crates/synthonyx-kit-compliance/tests/compliance_classification.rs` |
| Art. 30 | Records of processing — attribution | `OriginTrait`, `OriginSnapshot` on `AuditEntry` | `crates/synthonyx-kit-core/tests/compliance_origin.rs` |
| Art. 30 | Records of processing — first-entry hash | `AuditEntry.prev_hash` initialised to zero | `crates/synthonyx-kit-core/tests/compliance_audit_entry.rs` |
| Art. 32 | Security of processing — secret handling | `Secret<T>` (redacted Debug, no implicit serde, zeroize on drop) | `crates/synthonyx-kit-core/tests/compliance_secret.rs` |
| Art. 32 | Security of processing — password hashing | `Argon2Password` (Argon2id v19) | `crates/synthonyx-kit-password/tests/compliance_argon2.rs` |
| Art. 32 | Authentication integrity — typed errors not panics | `Argon2Password::verify` returns `PasswordError` | `crates/synthonyx-kit-password/tests/compliance_argon2.rs` |

## DORA — Digital Operational Resilience Act (EU 2022/2554)

| Article | Requirement (summary) | Kit primitive | Test file |
|---|---|---|---|
| Art. 9 | Audit-trail retention — round-trip and reopen | `FileAuditLogger`, `verify_chain` | `crates/synthonyx-kit-audit/tests/compliance_file_logger.rs` |
| Art. 9 | Audit-trail retention — durability across restarts | `FileAuditLogger::open` re-verifies chain | `crates/synthonyx-kit-audit/tests/compliance_file_logger.rs` |
| Art. 17 | Incident attribution — correlation propagation | `CorrelationId`, `SpanId`, hex round-trip | `crates/synthonyx-kit-core/tests/proptest_correlation.rs` |
| Art. 17 | Incident attribution — cross-service trace | W3C `traceparent` parser/formatter | `crates/synthonyx-kit-tracing/tests/compliance_w3c.rs` |
| Art. 17 | Incident attribution — robustness to malicious input | W3C parser never panics | `crates/synthonyx-kit-tracing/tests/proptest_w3c.rs` |
| Art. 17 | Incident timestamps — testable injectable clock | `TimeSource`, `MockClock`, `SystemClock` | `crates/synthonyx-kit-core/tests/compliance_time.rs` |
| Art. 17 | Origin kind in every dispatch | `OriginTrait::kind`, `BaseOrigin<P>` variants | `crates/synthonyx-kit-core/tests/compliance_origin.rs` |
| Art. 18 | Incident classification | `IncidentClass`, `Severity` | `crates/synthonyx-kit-compliance/tests/compliance_classification.rs` |
| Art. 19 | Major-incident reporting (24h clock) | `Severity::Critical` hook (Phase 2 reporting) | _Phase 2 — `synthonyx-kit-incident` crate._ |
| Art. 32 | Audit-log integrity — hash-chained entries | BLAKE3 chain in `FileAuditLogger` | `crates/synthonyx-kit-audit/tests/compliance_file_logger.rs` |
| Art. 32 | Audit-log integrity — concurrent writers | Mutex-guarded append, chain holds | `crates/synthonyx-kit-audit/tests/compliance_file_logger.rs` |
| Art. 32 | Audit-log integrity — tamper detection (any non-terminal position) | `verify_chain` property | `crates/synthonyx-kit-audit/tests/proptest_chain.rs` |
| Art. 32 | Audit-log integrity — deterministic encoding | `AuditEntry` serialises identically each call | `crates/synthonyx-kit-core/tests/compliance_audit_entry.rs` |

## NIS2 — Network and Information Systems Directive 2 (EU 2022/2555)

| Article | Requirement (summary) | Kit primitive | Test file |
|---|---|---|---|
| Art. 3 | Essential / important entity classification | `Nis2Class` | `crates/synthonyx-kit-compliance/tests/compliance_classification.rs` |
| Art. 21 | Risk-management measures — event severity taxonomy | `Severity` | `crates/synthonyx-kit-compliance/tests/compliance_classification.rs` |
| Art. 21 | Risk-management measures — auditable trail | `AuditLogger`, `AuditEntry` | `crates/synthonyx-kit-core/tests/compliance_audit_entry.rs` |
| Art. 23 | Incident notification — classification first | `IncidentClass` ordering by severity | `crates/synthonyx-kit-compliance/tests/compliance_classification.rs` |

## eIDAS 2.0 — Electronic Identification, Authentication and Trust Services (Reg. 2024/1183)

| Article | Requirement (summary) | Kit primitive | Test file |
|---|---|---|---|
| Art. 24a | Identity binding on every operation | `OriginTrait::principal`, `BaseOrigin::User/Service` | `crates/synthonyx-kit-core/tests/compliance_origin.rs` |

## ISO/IEC 27001 (Annex A controls touched)

| Control | Requirement (summary) | Kit primitive | Test file |
|---|---|---|---|
| A.5.34 | Privacy and PII protection | `Pii<T, C>` | `crates/synthonyx-kit-core/tests/compliance_pii.rs` |
| A.5.34 | Privacy and PII protection — secret material | `Secret<T>` | `crates/synthonyx-kit-core/tests/compliance_secret.rs` |
| A.8.5 | Secure authentication information | `Argon2Password` | `crates/synthonyx-kit-password/tests/compliance_argon2.rs` |
| A.8.15 | Logging — integrity of audit trail | BLAKE3-chained `FileAuditLogger` | `crates/synthonyx-kit-audit/tests/compliance_file_logger.rs` |
| A.8.16 | Monitoring — propagating correlation across calls | `TraceContext`, W3C propagation | `crates/synthonyx-kit-tracing/tests/compliance_w3c.rs` |

## ENISA cryptographic guidelines

| Guideline area | Requirement (summary) | Kit primitive | Test file |
|---|---|---|---|
| Password hashing | Argon2id v19 with library defaults | `Argon2Password::new` | `crates/synthonyx-kit-password/tests/compliance_argon2.rs` |
| Sensitive-material handling | No accidental logging or serialisation | `Secret<T>` static-asserts non-implementation of `Display` and `Serialize` | `crates/synthonyx-kit-core/tests/compliance_secret.rs` |
| Trace identifiers | 128-bit trace id, 64-bit span id, hex transport | `CorrelationId`, `SpanId` hex round-trip | `crates/synthonyx-kit-core/tests/proptest_correlation.rs` |

## Phase 2 deferrals (no test file in current version)

These rows are tracked as planned coverage; their test files will be added
alongside the Phase 2 backends and incident-reporting crate.

| Regulation | Article | Requirement | Implementing crate (planned) |
|---|---|---|---|
| GDPR | Art. 5(1)(e) | Retention sweep execution | `synthonyx-kit-storage-postgres` (Phase 2) |
| GDPR | Art. 17 | Erasure fan-out across `Erasable` RTMs | `synthonyx-kit-compliance` extended in Phase 2 |
| GDPR | Art. 32 | Encryption at rest | `synthonyx-kit-storage-postgres` (Phase 2) |
| DORA | Art. 19 | Major-incident 24h reporting | `synthonyx-kit-incident` (Phase 2) |
| DORA | Art. 32 | Key rotation for audit signing | `synthonyx-kit-storage-postgres` (Phase 2) |

## Adding a new compliance test

1. Author the test under `crates/<crate>/tests/compliance_<topic>.rs` (or
   `proptest_<topic>.rs` for property-based coverage).
2. Add a row to the appropriate table above. The `Test file` cell must
   contain the full repo-relative path inside backticks.
3. Run `cargo test -p compliance-matrix-test`. It will fail until the
   matrix and the test files agree.
