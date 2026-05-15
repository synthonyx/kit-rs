# Synthonyx Kit for Rust

A development kit for rapidly building **enterprise-grade,
EU-compliance-ready** (web) services in Rust around a modular runtime
architecture.

**This project is under active development and not yet production ready.**

## What it is

Services built with the kit are composed from **Runtime Modules (RTMs)**.
Each RTM declares its `Config`, errors, events, storage shape, and
lifecycle hooks against a small kernel of traits; a runtime composer
(hand-written today, macro-generated in a future release) wires the
RTMs into a deployable service. The same RTM can run embedded in a
monolithic service or split out as a sidecar behind an API adapter.

The kit targets services subject to EU regulations including **DORA, GDPR,
NIS2, and eIDAS 2.0**. Compliance primitives — tamper-evident audit log,
secret redaction, PII type tagging, monotonic time injection, W3C trace
propagation — ship as part of the kernel so they cannot be retrofitted.

See [`docs/standards.md`](docs/standards.md) for the regulatory frameworks
the kit explicitly addresses.

## Workspace

| Crate | Purpose |
|---|---|
| `synthonyx-kit` | Facade — re-exports the curated public API of the other crates. |
| `synthonyx-kit-core` | Kernel traits: `Get`, `Config`, `Dispatch`, `Origin`, `Event`, `Hooks`, `Time`, `Secret`, `Pii`, `AuditLogger`, `CorrelationId`. |
| `synthonyx-kit-primitives` | `param!`, `env_param!`, and `ConstU8..ConstBool` parameter-source macros and types. |
| `synthonyx-kit-storage` | Backend-agnostic storage traits with compliance hooks (encryption-at-rest, retention, erasure). |
| `synthonyx-kit-audit` | Concrete audit sinks: `FileAuditLogger` (append-only, BLAKE3-chained, tamper-evident) and `TracingAuditLogger`. |
| `synthonyx-kit-compliance` | EU regulatory taxonomy: `DataSubjectId`, `GdprCategory`, `IncidentClass`, `Nis2Class`, `Erasable`. |
| `synthonyx-kit-tracing` | W3C `traceparent` propagation and a Tokio task-local trace context. |
| `synthonyx-kit-password` | `PasswordChecker` trait and the `Argon2Password` reference implementation. |
| `synthonyx-kit-macros` | Procedural macros. Reserved for a future release; empty in the current version. |

## Using this crate

Add the facade as a dependency and pull in the features you need:

```toml
[dependencies]
synthonyx-kit = { version = "0.2", features = ["audit", "compliance", "tracing", "password"] }
```

Or depend on individual member crates directly for fine-grained control over
your dependency tree:

```toml
[dependencies]
synthonyx-kit-core = "0.2"
synthonyx-kit-audit = "0.2"
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`docs/release-process.md`](docs/release-process.md).

## Licensing

Copyright (c) 2024-2026, [Synthonyx Technologies Ltd](https://synthonyx.com).

This kit is dual-licensed under the terms of the Apache License, Version 2.0
and the MIT license. Choose the one that best fits your needs.
