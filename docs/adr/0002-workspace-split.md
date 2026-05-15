# 0002 — Workspace split into nine crates

**Status:** Accepted

## Context

The kit ranges from zero-dependency kernel traits to crates that pull in
`tokio`, `http`, `tracing`, `blake3`, `argon2`, etc. A monolithic crate
would force every consumer to compile every transitive dependency. In
particular, sidecars that need only `-core` + a transport adapter would
pull in argon2's cryptographic machinery and BLAKE3 for no reason.

We considered three layouts: monolithic single crate, single crate with
feature flags, and a workspace of small crates. The user-confirmed
direction in the plan is the workspace of small crates.

## Decision

Nine published crates plus one internal test crate. Dependency direction:

```
synthonyx-kit (facade) → all members
synthonyx-kit-{primitives,storage,audit,compliance,tracing,password} → -core
synthonyx-kit-compliance → -storage
synthonyx-kit-core → no other kit crate
synthonyx-kit-macros → no kit crate (proc-macro only; populated in Phase 3)
compliance-matrix-test → internal; verifies docs/compliance.md stays in sync
```

The facade `synthonyx-kit` is the recommended dependency for application
consumers; member crates are exposed for fine-grained dependency control
(e.g. a sidecar that only needs `-core` + a Phase 2 transport).

## Consequences

**Positive.** Sidecars and small services can pull only what they need.
Phase 2 storage backends (Postgres, SQLite, Redis) will land as separate
crates without touching the kernel. The facade absorbs the discoverability
cost of multiple crates.

**Negative.** More `Cargo.toml` files to maintain. More `cargo publish`
steps per release (mitigated by [`docs/release-process.md`](../release-process.md)).
Workspace-level dependency pinning via `[workspace.dependencies]` keeps
versions consistent across members.

**Why not single crate with features?** Feature combinatorics get painful
fast when storage backends and API surfaces multiply in Phase 2. Proc-macro
crates also cannot coexist with library crates in a single Cargo package, so
`-macros` would need to be split out anyway.
