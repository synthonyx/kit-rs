# synthonyx-kit-audit

Concrete audit-log sinks for the Synthonyx Kit. The `AuditLogger` trait
and `AuditEntry` type itself live in
[`synthonyx-kit-core`](../synthonyx-kit-core) — see
[ADR 0007](../../docs/adr/0007-config-binds-auditlogger.md) for why.

## Contents

- `FileAuditLogger` — append-only, BLAKE3-chained, tamper-evident,
  std-only. Each entry's `prev_hash` is the BLAKE3-32 hash of the
  previous serialised line. Reopening verifies the chain end-to-end.
- `TracingAuditLogger` — emits entries via the `tracing` crate; intended
  for development and observability. **Not** sufficient as the sole sink
  for DORA-regulated retention; pair with a persistent sink.
- `verify_chain(path)` — standalone verifier returning the final running
  hash or `AuditError::ChainBroken`.

```rust
use synthonyx_kit_audit::{FileAuditLogger, verify_chain};

let logger = FileAuditLogger::open("/var/log/myservice/audit.jsonl")?;
// ... logger.record(entry) ...
verify_chain("/var/log/myservice/audit.jsonl")?;
# Ok::<(), synthonyx_kit_core::AuditError>(())
```

The terminal-entry tamper-detection limitation is documented in
[ADR 0008](../../docs/adr/0008-tamper-evident-audit-chain.md).
