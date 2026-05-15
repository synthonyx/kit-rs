# 0007 — `AuditLogger` trait lives in `-core`, not `-audit`

**Status:** Accepted

## Context

The kernel `Config` trait has an associated type bound on `AuditLogger`:

```rust
pub trait Config: 'static {
    // ...
    type Audit: AuditLogger;
}
```

Natural homes for the `AuditLogger` trait:

1. **`-audit`** — semantically, the audit crate owns audit. This is the
   intuitive placement.
2. **`-core`** — alongside `Config` itself.

Option 1 creates a circular dependency: `-core` depends on `-audit` (for
the trait bound), and `-audit` depends on `-core` (for `AuditEntry`'s
`CorrelationId`, `OriginKind`, `UnixNanos` fields). Cargo rejects cyclic
dependencies between published crates.

Solutions considered for breaking the cycle:

- **Move shared types from `-core` to a deeper crate** (e.g.
  `synthonyx-kit-types`). Adds another crate without solving the abstract
  problem.
- **Use a smaller shared trait crate** (`synthonyx-kit-traits`). Same.
- **Drop the `Audit` bound from `Config`**. Loses compile-time guarantee
  that every runtime wires an auditor.
- **Move the trait to `-core`; keep concrete sinks in `-audit`**.

## Decision

The `AuditLogger` trait, `AuditEntry`, `AuditValue`, `Outcome`,
`OriginSnapshot`, and `AuditError` all live in `-core` (under
`synthonyx_kit_core::audit`). Concrete sinks — `FileAuditLogger`,
`TracingAuditLogger`, and `verify_chain` — live in `-audit`.

`-audit` continues to depend on `-core` for the trait and entry types.
`-core` depends on no other kit crate.

## Consequences

**Positive.** No cycle. `Config::Audit` can bound on the trait directly.
`-core` stays the single root of the crate graph. The trait is small
(one method) so it doesn't bloat `-core`.

**Negative.** Slight conceptual coupling: `-core` knows about "audit log"
as a concept even though it doesn't implement any sink. We consider this
acceptable — audit is a first-class compliance primitive, not an optional
add-on.

**Alternative we did not take.** Some kits in this space define a tiny
`audit-traits` crate that both `-core` and `-audit` depend on. We
considered this and rejected it: an extra published crate for one trait
is more cost than the conceptual blur of having the trait in `-core`.
