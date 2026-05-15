# Architecture

A reading order for contributors landing in this codebase. The kit is small
enough to fit in one head; this document gives the mental model so you can
read the source with intent.

## The one-line summary

A Synthonyx Kit runtime is **a struct that implements `Config`** for one or
more **Runtime Modules (RTMs)**. RTMs are independent units of business
logic; the runtime composer wires them to shared infrastructure (audit, time,
storage, events, origin) through associated types on `Config`. Dispatches
flow from an inbound API edge, through `Dispatch::call(origin)`, into RTM
logic, out to storage / events / audit, and back as a typed result.

## Crate graph

```
        ┌──────────────────────────────────────────┐
        │              synthonyx-kit               │  ← facade (feature-gated re-exports)
        └────────────────────┬─────────────────────┘
                             │
   ┌──────────┬──────────┬───┴───────┬──────────┬──────────┬──────────┐
   ▼          ▼          ▼           ▼          ▼          ▼          ▼
 -core    -primitives  -storage    -audit  -compliance -tracing  -password
   │                       │          │         │          │          │
   └───────────┬───────────┘          │         │          │          │
               │   (depends on -core) │         │          │          │
               ▼                      │         │          │          │
            -storage ──────────────► -compliance         (deps on -core)
```

Rules:
- `-core` depends on no other kit crate.
- Every other crate depends on `-core` (sometimes `-storage` too).
- `-macros` is a placeholder until Phase 3 (`compose_runtime!`).
- The facade `synthonyx-kit` re-exports curated public APIs under feature
  flags; downstream users can also depend on member crates directly.

## The `Config` seam

```rust
pub trait Config: 'static {
    const MODULE: &'static str;
    type RuntimeEvent: From<Self::Event>;
    type Event: synthonyx_kit_core::Event;
    type Error: std::error::Error + Send + Sync + 'static + Into<DispatchError>;
    type Origin: OriginTrait;
    type Time: TimeSource;
    type Audit: AuditLogger;
}
```

This is the *only* mandatory contract every RTM speaks. Each associated
type is supplied by the runtime composer (a hand-written `impl Config for
MyRuntime` today; `compose_runtime!`-generated in Phase 3). RTMs add
their own extension traits — typically `MyRtmConfig: Config` with
extra associated types like `type Users: StorageMap<...>;`.

## Dispatch flow

```
inbound request (HTTP / gRPC / sidecar IPC)
       │
       ▼
build an Origin (Origin::User { principal, correlation_id })
       │
       ▼
RTM call enum  ─────►  Dispatch::call(&self, origin)  ─────►  Result<Output, DispatchError>
                                │
                ┌───────────────┼──────────────────────────┐
                ▼               ▼                          ▼
            T::Time::now()  T::Audit::record(...)    T::RuntimeEvent
                                                        emitted
```

Every dispatch:
1. Carries an `Origin` (who triggered this, with a `CorrelationId`).
2. Returns `Result<_, DispatchError>` — typed errors compose via `Into`.
3. May call `T::Audit::record(AuditEntry { ... })` to append to the audit log.
4. May emit `T::Event` instances that aggregate into `T::RuntimeEvent`.
5. Reads time only through `T::Time` (so tests can inject `MockClock`).

## Compliance flow

```
Secret<T>                              Pii<T, C>
   │                                       │
   │ Debug → redacted             Debug → "Pii<category>(<redacted>)"
   │ Serialize  → NOT IMPLEMENTED         transparent serde round-trip
   │ Drop       → zeroize                 storage backends (Phase 2)
   │                                      route on `C::CATEGORY`
   ▼                                       │
authentication path                        ▼
   │                                  GDPR Art. 17 erasure (Phase 2)
   ▼                                  via `Erasable` fan-out
Argon2Password (Argon2id v19)
```

```
CorrelationId  ─────►  TraceContext  ─────►  W3C `traceparent` HTTP header
   ▲                       │
   │                       ▼
   │                  task_local!  ◄──── tokio::task::scope (Phase 2 entry point)
   │
OriginTrait::correlation_id()
   │
   ▼
AuditEntry.correlation_id  ────►  BLAKE3-chained file
                                  (FileAuditLogger)
```

## Lifecycle hooks

RTMs may implement `Hooks` to plug into service lifecycle:

```
on_boot      → once per process start, after Config is wired
   ▼
   ... service serves traffic ...
   ▼
on_idle      → periodic, between requests; deadline-bounded
on_migrate   → invoked by the migration orchestrator (not every boot)
   ▼
on_shutdown  → once during graceful shutdown (LIFO order across RTMs)
```

All four methods are defaulted to `Ok(())`, so RTMs implement only what they
need.

## Sidecar pattern

A sidecar in this kit is **the same RTM, just hosted in a separate
process** behind an API adapter. Phase 2 adds `SidecarServer<R>` /
`SidecarClient<R>` transports; Phase 1 nails the contract so the same RTM
compiles unchanged whether it runs embedded or remote.

```
+--------------------+    mTLS / unix-socket / NATS    +-------------------------+
| main service       | <─────────────────────────────► | sidecar service         |
|  - composes:       |                                  |  - composes:           |
|    AuthRtm         |                                  |    PaymentsRtm         |
|    UsersRtm        |                                  |  (own audit log,       |
|    AuditRtm        |                                  |   own storage,         |
|  - SidecarClient<  |                                  |   own Config impl)     |
|      PaymentsRtm>  |                                  |                        |
+--------------------+                                  +-------------------------+
```

The W3C trace context propagates across the boundary so a single logical
operation can be reconstructed end-to-end.

## What lives where

| Concern | Where | Why |
|---|---|---|
| Get<T> parameter pattern | `-core::get` / `-primitives` | Used by every RTM; primitives crate provides the parameter-source macros. |
| Per-RTM contract | `-core::config::Config` | The trait every RTM extends. |
| Dispatch entry point | `-core::dispatch` | Sync (`Dispatch`) and async (`DispatchAsync` + `Send`-bounded variant). |
| Who triggered the dispatch | `-core::origin` | `OriginTrait`, `BaseOrigin<P>`, `OriginKind`. |
| What time is it | `-core::time` | `TimeSource`, `SystemClock`, `MockClock`. |
| Correlation across calls | `-core::correlation` (types) + `-tracing` (HTTP propagation) | Types are zero-dep; propagation pulls in `http` + `tokio`. |
| Sensitive material | `-core::secret` | `Secret<T: Zeroize>`. |
| Personal data tagging | `-core::pii` | `Pii<T, C: PiiCategory>`. |
| Audit log | `-core::audit` (trait + entry types) + `-audit` (impls) | Trait in `-core` to avoid `-core` ↔ `-audit` cycle via `Config::Audit`. |
| Errors | `-core::error::DispatchError` | thiserror-derived; per-RTM errors compose via `Into`. |
| Storage abstraction | `-storage` | Backend-agnostic traits + compliance hooks (encryption / retention / erasure). |
| Regulatory taxonomy | `-compliance` | `DataSubjectId`, `Severity`, `IncidentClass`, `Nis2Class`, `Erasable`. |
| Password hashing | `-password` | `PasswordChecker`, `Argon2Password`. |
| Macros (Phase 3) | `-macros` | Empty in Phase 1; populated with `compose_runtime!` later. |

## When in doubt

- **"Where should this go?"** — if it has no dependencies, `-core`. If it
  needs `tokio`/`tracing`/`http`, `-tracing` (or a Phase 2 transport crate).
  If it's an implementation of an existing trait, the implementing crate.
- **"Should this be async?"** — if it can block on I/O, yes (`async fn` +
  `trait_variant::make` for `dyn Send`). If it's pure computation, no.
- **"Should this be feature-gated?"** — yes if it pulls in a heavy dep
  (`serde`, `tokio`, `tracing-subscriber`). No if it's already a hard
  dependency anyway.
