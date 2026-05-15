# synthonyx-kit

The facade crate of the Synthonyx Kit. Re-exports curated APIs from every
member crate under feature flags so application consumers can have a
single dependency.

```toml
[dependencies]
synthonyx-kit = { version = "0.2", features = ["audit", "tracing", "password"] }
```

```rust
use synthonyx_kit::{Config, Get, param};
use synthonyx_kit::audit::TracingAuditLogger;

param!(MaxRetries: u32 = 5u32);
assert_eq!(MaxRetries::get(), 5);
```

For fine-grained dependency control, depend on the member crates
(`synthonyx-kit-core`, `synthonyx-kit-audit`, etc.) directly.

See [`docs/architecture.md`](../../docs/architecture.md) for the bigger
picture and [`docs/writing-an-rtm.md`](../../docs/writing-an-rtm.md) for
the first-RTM tutorial.
