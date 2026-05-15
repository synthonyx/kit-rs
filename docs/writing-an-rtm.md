# Writing a Runtime Module (RTM)

A walkthrough of the smallest interesting RTM you can write today, by hand,
without the Phase 3 macro DSL. The end product is `examples/manual_runtime.rs`
in the `synthonyx-kit` crate; run it with:

```sh
cargo run --example manual_runtime -p synthonyx-kit
```

By the end of this document you'll have:
1. Defined an RTM's `Event` and `Error` types.
2. Defined a `Config` extension trait for the RTM.
3. Implemented a stateful RTM struct (`UsersRtm<T>`) with a typed dispatch
   method that records an audit entry and emits an event.
4. Composed it into a `Runtime` struct that implements `Config`.
5. Called the dispatch with a fully-wired `Origin` and observed the
   audit/event flow.

## Goal: a tiny `users` RTM

The RTM exposes one dispatch — `register_user` — that:
- Takes an origin (who is registering) and a user name.
- Validates the name is non-empty.
- Records an audit entry attributing the operation to the origin.
- Emits a `UserRegistered` event.
- Returns `Result<UsersEvent, DispatchError>`.

It deliberately omits storage to keep the example self-contained; in a real
RTM you would add `type Users: StorageMap<Key = UserId, Value = User>;` to
the `Config` extension trait and persist the registration.

## Step 1 — the event

Every emitting RTM defines its own `Event` type and implements
`synthonyx_kit_core::Event` for it. The event identifies its module, names
the variant, and tags a severity that audit and incident-reporting sinks
dispatch on.

```rust
use synthonyx_kit_core::{Event, Severity};

#[derive(Debug)]
pub enum UsersEvent {
    UserRegistered { name: String },
}

impl Event for UsersEvent {
    fn module(&self) -> &'static str { "users" }
    fn name(&self) -> &'static str {
        match self {
            UsersEvent::UserRegistered { .. } => "user_registered",
        }
    }
    fn severity(&self) -> Severity { Severity::Info }
}
```

## Step 2 — the error

Per-RTM errors are `thiserror`-derived and compose into the kit-wide
`DispatchError` via `Into`. The `module` field on `DispatchError::Module`
ties the error back to the RTM in audit / observability output.

```rust
use synthonyx_kit_core::DispatchError;

#[derive(Debug, thiserror::Error)]
pub enum UsersError {
    #[error("user name must not be empty")]
    EmptyName,
}

impl From<UsersError> for DispatchError {
    fn from(e: UsersError) -> Self {
        DispatchError::module("users", e)
    }
}
```

## Step 3 — the Config extension trait

Every RTM extends the kernel's `Config` trait with its own. This is where
RTM-specific associated types — storage, parameter types, RTM-local config
constants — live. Our `users` RTM doesn't need any extra associated types
yet; we just nail down the `Event` and `Error`:

```rust
use synthonyx_kit_core::Config;

pub trait UsersConfig: Config<Event = UsersEvent, Error = UsersError> {}
```

A real RTM would add things like:

```rust
pub trait UsersConfig: Config<Event = UsersEvent, Error = UsersError> {
    /// Maximum length of a user name.
    type MaxNameLen: synthonyx_kit_core::Get<u32>;
    /// Backing storage.
    type Users: synthonyx_kit_storage::StorageMap<Key = UserId, Value = User>;
}
```

## Step 4 — the RTM struct

The RTM struct holds whatever it needs to operate on each call — for us,
that's the audit sink and the time source. Both come from `Config`
associated types, so the same RTM code compiles unchanged against any
runtime that wires those types.

```rust
pub struct UsersRtm<T: UsersConfig> {
    audit: T::Audit,
    time: T::Time,
}

impl<T: UsersConfig> UsersRtm<T> {
    pub fn new(audit: T::Audit, time: T::Time) -> Self {
        Self { audit, time }
    }
}
```

## Step 5 — the dispatch method

The dispatch validates input, records an audit entry, and returns the
event. Three things to notice:

- We pull the wall-clock timestamp from `self.time` — never from
  `std::time::SystemTime::now()` directly. The clippy `disallowed-methods`
  lint enforces this.
- We pull the correlation id from the `Origin`. Every audit entry carries
  the same id as the inbound dispatch so a single logical operation can be
  stitched across logs.
- We compose `UsersError` into `DispatchError` via `?` — the `Into` impl
  from Step 2 makes this transparent.

```rust
use std::borrow::Cow;
use std::collections::BTreeMap;

use synthonyx_kit_core::{
    AuditEntry, AuditLogger, OriginSnapshot, OriginTrait, Outcome, TimeSource,
};

impl<T: UsersConfig> UsersRtm<T> {
    pub fn register_user(
        &self,
        origin: T::Origin,
        name: String,
    ) -> Result<UsersEvent, DispatchError> {
        if name.is_empty() {
            return Err(UsersError::EmptyName.into());
        }

        let entry = AuditEntry {
            timestamp: self.time.now(),
            correlation_id: origin.correlation_id(),
            module: Cow::Borrowed(<T as Config>::MODULE),
            action: Cow::Borrowed("register_user"),
            origin: OriginSnapshot {
                kind: origin.kind(),
                principal: None,
                service: None,
            },
            outcome: Outcome::Success,
            subject: Some(name.clone()),
            fields: BTreeMap::new(),
            prev_hash: [0u8; 32],
        };
        self.audit
            .record(entry)
            .expect("audit recording must not fail in this example");

        Ok(UsersEvent::UserRegistered { name })
    }
}
```

> **Note — `Dispatch` trait.** For a single-method RTM, calling
> `users.register_user(...)` directly is the simplest pattern. The
> `Dispatch<T>` trait exists for code paths that need to *serialise* a
> dispatch (e.g. enqueue it for later execution, or route it from a
> macro-generated runtime composer). Phase 3's `compose_runtime!` will
> generate a `UsersCall` enum + `Dispatch` impl that fans out to the typed
> methods you wrote above. In Phase 1, you can do the same by hand, but it
> is not required for the dispatch to function.

## Step 6 — the runtime composer

The runtime composer is a concrete type that implements `Config` (and any
RTM-specific `*Config` extension traits). In Phase 3 this will be macro-
generated; today you write it yourself.

```rust
use synthonyx_kit_audit::TracingAuditLogger;
use synthonyx_kit_core::{BaseOrigin, SystemClock};

struct MyRuntime;

impl Config for MyRuntime {
    const MODULE: &'static str = "users";
    type RuntimeEvent = UsersEvent;     // single-RTM runtime; aggregate enum unnecessary
    type Event = UsersEvent;
    type Error = UsersError;
    type Origin = BaseOrigin<String>;
    type Time = SystemClock;
    type Audit = TracingAuditLogger;
}

impl UsersConfig for MyRuntime {}
```

When you compose multiple RTMs, `RuntimeEvent` becomes an aggregate enum
that each module's `Event` flows into via `From`. Phase 3's macro will
generate this enum automatically.

## Step 7 — wiring and calling

```rust
use synthonyx_kit_core::CorrelationId;

fn main() {
    let users = UsersRtm::<MyRuntime>::new(TracingAuditLogger, SystemClock);

    let origin = BaseOrigin::User {
        principal: "user-42".to_string(),
        correlation: CorrelationId([42u8; 16]),
    };

    let event = users
        .register_user(origin, "alice".to_string())
        .expect("non-empty name registers successfully");

    println!("Emitted event: {event:?}");
}
```

Running this prints (to `stderr`, via `tracing`):
```
INFO synthonyx::audit: {"timestamp":...,"correlation_id":[...],"module":"users","action":"register_user", ...}
Emitted event: UserRegistered { name: "alice" }
```

The audit entry is structured JSON, with the correlation id, principal
attribution, outcome, and timestamp ready for downstream collection.

## What you got for free

Even this tiny RTM gets a number of compliance properties out of the box:

- **Audit trail with attribution** (GDPR Art. 30, DORA Art. 17): every
  dispatch records an `OriginSnapshot` and a `CorrelationId`.
- **Deterministic timestamps** (DORA Art. 17): `T::Time` is injected from
  `Config`; tests can swap in `MockClock`.
- **No accidental leakage of sensitive material** (GDPR Art. 32): had you
  taken a `Secret<String>` as the user's password, it would have refused
  to serialise into the audit log and panicked-by-Debug only with
  `<redacted>`.
- **Typed errors that compose** (DORA Art. 32 — structured incident
  reporting): `UsersError` → `DispatchError::Module` keeps the module
  attribution; downstream tooling can dispatch on it without string
  matching.

## What to read next

- `docs/architecture.md` — the bigger mental model, including hooks, the
  sidecar pattern, and the crate graph.
- `docs/compliance.md` — which test files prove which regulatory contracts.
- `docs/adr/` — the design decisions behind the kit, with rationale.
- The full source of `examples/manual_runtime.rs` — the same code as this
  tutorial, in one compilable file.
