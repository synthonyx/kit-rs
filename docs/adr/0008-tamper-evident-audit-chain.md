# 0008 — BLAKE3-chained audit log

**Status:** Accepted

## Context

DORA Art. 32 requires audit-log integrity: an attacker (insider or
otherwise) with write access to the persisted log must not be able to
alter past entries undetectably. Options considered:

1. **Per-entry HMAC**. A keyed MAC over each entry's bytes, with the key
   stored separately. Detects tamper of any entry, but requires key
   management (rotation, escrow) and a secret to verify.
2. **BLAKE3 hash chain**. Each entry's `prev_hash` field is the BLAKE3
   hash of the previous entry's canonical encoding. Verification needs
   only the log and the constant-zero genesis hash — no secret. Detects
   tamper at any non-terminal position.
3. **Merkle tree**. Random-access tamper detection with O(log n) proofs.
   Powerful but operationally complex (rebuild on append; balance
   maintenance; root publication).
4. **Per-entry signatures**. Strong non-repudiation; requires a signing
   identity (HSM / KMS).

For Phase 1 we want a zero-config, zero-key, std-only sink that ships
with the audit crate. Options 1 and 4 require key management that's
inappropriate for a kernel default. Option 3 is more complexity than
warranted.

## Decision

`FileAuditLogger` writes one JSON entry per line. Each entry carries a
`prev_hash` field equal to BLAKE3-32 of the previous line's bytes
(excluding the trailing newline). The first entry's `prev_hash` is all
zeros.

Encoding is `serde_json::to_string`, which is deterministic per the
field order in the struct definition; the `fields` map is a `BTreeMap` so
its keys serialise in sorted order. This produces a canonical encoding
without an external canonicalisation library.

The logger overwrites the caller-supplied `prev_hash` field on every
`record()` call. Callers therefore do not need to track the running hash;
they can leave the field zero.

`verify_chain(path)` walks the file from the start, recomputing the
expected `prev_hash` for each entry. On mismatch it returns
`AuditError::ChainBroken { expected, actual }` with both hex-encoded
hashes. On reopen, `FileAuditLogger::open` invokes `verify_chain` to
restore the running hash and reject already-corrupted logs.

## Consequences

**Positive.** Zero key management. Zero external dependencies beyond
`blake3` and `serde_json`. The chain is verifiable by anyone with read
access to the log; no shared secret is required. Tamper detection
extends across the whole file: a flip of any byte in any non-terminal
entry breaks the next entry's `prev_hash` check.

**Negative — known limitation.** The terminal entry cannot be detected as
tampered, because there is no successor whose `prev_hash` would expose
it. This is a property of any forward-only hash chain. Tests document
this explicitly
([`tamper_in_non_terminal_entry_is_detected`](../../crates/synthonyx-kit-audit/tests/proptest_chain.rs)).
Mitigations for callers who need terminal-entry coverage: (a) periodically
write a "checkpoint" entry whose only purpose is to bind the previous
real entry; (b) Phase 2 may add signed checkpoints via a KMS hook.

**Negative — JSON-encoding stability.** Although `serde_json` is
deterministic for our struct shape today, a future serde version could
in principle change whitespace handling. We pin canonicalisation to "no
whitespace, struct-field-declaration order, sorted-key BTreeMap" via the
test
[`art_32_serialises_deterministically`](../../crates/synthonyx-kit-core/tests/compliance_audit_entry.rs).

**Forward path.** Phase 2's `synthonyx-kit-storage-postgres` will offer a
table-backed audit sink with the same chain semantics. A future ADR will
cover optional per-entry signing for non-repudiation when a KMS is
available.
