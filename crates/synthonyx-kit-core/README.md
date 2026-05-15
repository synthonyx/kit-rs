# synthonyx-kit-core

The kernel of the Synthonyx Kit. Defines every trait that a Runtime Module
(RTM) must speak, plus the compliance primitives the kit treats as
non-negotiable foundations.

This is the only crate every other kit crate depends on; it has no
dependency on any other kit crate itself.

## Contents

- `Get<T>` — type-level parameter pattern.
- `Config` — per-RTM contract with `RuntimeEvent`, `Event`, `Error`,
  `Origin`, `Time`, `Audit` associated types.
- `Dispatch<T: Config>` / `DispatchAsync<T: Config>` — sync and async
  dispatch entry points (`DispatchAsync` ships in a `Send`-bounded variant
  via `trait_variant`).
- `OriginTrait` / `BaseOrigin<P>` / `OriginKind` — who triggered the call.
- `Event` / `Severity` / `EventBus` — domain event emission.
- `Hooks` — `on_boot`, `on_shutdown`, `on_migrate`, `on_idle` lifecycle.
- `DispatchError` — kit-wide error, thiserror-derived, composes per-RTM
  errors via `Into`.
- `Secret<T: Zeroize>` — redacted, zeroize-on-drop wrapper; does not
  implement `Serialize` / `Display`.
- `Pii<T, C: PiiCategory>` — type-tagged personal data; redacted in
  `Debug` but transparent in serde.
- `TimeSource` + `SystemClock` + `MockClock` — injectable clocks.
- `CorrelationId` / `SpanId` — 128-/64-bit identifiers with hex display.
- `AuditLogger` / `AuditEntry` / `AuditValue` / `Outcome` /
  `OriginSnapshot` — the audit trait and entry shape; concrete sinks live
  in `synthonyx-kit-audit`.

```rust
use synthonyx_kit_core::{Get, Secret};
use std::fmt::Write;

let pw = Secret::new(String::from("hunter2"));
let mut s = String::new();
write!(s, "{pw:?}").unwrap();
assert_eq!(s, "Secret(<redacted>)");
```

See [`docs/architecture.md`](../../docs/architecture.md) and the ADR
series under [`docs/adr/`](../../docs/adr/) for design rationale.
