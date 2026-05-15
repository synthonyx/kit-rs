# 0001 — Modular runtime architecture

**Status:** Accepted

## Context

The kit targets enterprise services in regulated domains (DORA, GDPR,
NIS2). These services share three structural needs:

1. **Composability.** A single service typically combines several
   independent units of business logic (user management, billing,
   identity, audit). Each unit has its own data shape, error model,
   and lifecycle, but all must be observable through one consistent
   runtime.
2. **Consistent infrastructure.** Every unit needs access to the same
   audit sink, time source, origin propagation, and error composition.
   Re-implementing these per unit is both wasteful and a compliance
   hazard (audit gaps appear where one module forgets to record).
3. **Compliance-by-construction.** Audit log, secret redaction, PII
   tagging, monotonic time, and correlation IDs must be available from
   the kernel — once a unit is written, retrofitting them is painful.

The architecture must therefore push isolation up to the per-unit
boundary while pulling cross-cutting concerns down into a single
kernel.

## Decision

Independent **RTMs** (Runtime Modules), each declaring:

- A per-module `Config` trait that extends the kernel `Config` with
  any RTM-specific associated types (storage shape, parameter
  constants).
- A typed `Event` enum implementing the kernel `Event` trait.
- A typed `Error` enum that composes into the kit-wide `DispatchError`
  via `Into`.
- A stateful RTM struct (`MyRtm<T: MyRtmConfig>`) holding the audit
  sink and time source projected through `Config`.
- Dispatchable operations as typed methods on the RTM struct (and, in
  Phase 3, also reflected as variants of a `Call` enum implementing
  the `Dispatch<T>` trait).

A **runtime composer** is a single concrete type that implements every
`Config` trait used by the embedded RTMs. The composer is the one and
only place that picks concrete `Time`, `Audit`, `Origin`, `Storage`
backends. In Phase 1 the composer is hand-written; in Phase 3 the
`compose_runtime!` procedural macro generates it from a declarative
RTM list.

## Consequences

**Positive.**

- RTMs are independently testable: a unit test for `MyRtm<TestRuntime>`
  needs nothing more than a stub `Config` impl and a `MockClock`.
- Cross-cutting compliance (audit, secret handling, correlation
  propagation) lives in `synthonyx-kit-core` and `-audit`; every RTM
  inherits it for free.
- The runtime composer is the one place to read when auditing a
  deployment's concrete configuration — "what time source does this
  service use?" has exactly one answer per binary.
- The sidecar pattern falls out of the architecture: a sidecar runs
  the same RTM behind a transport adapter, with its own runtime
  composer choosing potentially different audit/storage backends.

**Negative.**

- The associated-type-heavy `Config` pattern has a learning curve. A
  contributor's first RTM is more verbose than the same logic written
  as a free-standing module. [`docs/writing-an-rtm.md`](../writing-an-rtm.md)
  mitigates this with a step-by-step tutorial.
- Without Phase 3's macro, multi-RTM runtime composers can be
  boilerplate-heavy. This is a deliberate trade-off: Phase 1 keeps
  the macro layer out so the kernel can stabilise first; Phase 3
  adds the ergonomics on top of an already-tested foundation.

**Forward path.** Phase 3's `compose_runtime!` will generate the
runtime struct, the aggregate `RuntimeError`/`RuntimeEvent` enums,
the per-RTM `Config` impls, and the composed `Hooks` impl from a
declarative module list. Hand-written composers remain supported
indefinitely because the macro emits exactly the same code shape a
human would.
