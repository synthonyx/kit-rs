# Architecture Decision Records

These records capture the load-bearing design decisions for the kit. Each
ADR has a stable number; superseded records stay in place with a pointer to
their replacement.

| # | Title | Status |
|---|---|---|
| [0001](0001-modular-runtime-architecture.md) | Modular runtime architecture | Accepted |
| [0002](0002-workspace-split.md) | Workspace split into nine crates | Accepted |
| [0003](0003-afit-trait-variant.md) | `async fn` in trait + `trait_variant` for async dispatch | Accepted |
| [0004](0004-serde-codec.md) | Serde-based `Codec` trait, bincode2 default | Accepted |
| [0005](0005-secret-and-pii-redaction.md) | `Secret<T>` and `Pii<T, C>` redaction patterns | Accepted |
| [0006](0006-cow-static-str-in-audit-entry.md) | `Cow<'static, str>` for `AuditEntry::module` and `action` | Accepted |
| [0007](0007-config-binds-auditlogger.md) | `AuditLogger` trait lives in `-core` | Accepted |
| [0008](0008-tamper-evident-audit-chain.md) | BLAKE3-chained audit log | Accepted |

## Adding a new ADR

Use the next available number. Follow the template: Status / Context /
Decision / Consequences. Keep each section under a page. Update this index.
