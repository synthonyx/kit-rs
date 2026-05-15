# Glossary

Term and acronym reference for contributors landing in this codebase.

## Kit terms

| Term | Definition |
|---|---|
| **RTM** (Runtime Module) | A unit of business logic. Declares an `Event`, `Error`, `Config` extension trait, dispatchable operations, and storage shape. |
| **Runtime composer** | The code (hand-written today, macro-generated in Phase 3) that wires individual RTMs into a deployable service by implementing every `Config` trait on a concrete runtime type. |
| **Runtime** | The struct that aggregates RTMs and implements every RTM's `Config` trait. One process typically runs one runtime. |
| **Kernel** | The crate `synthonyx-kit-core`. The minimal set of traits an RTM must speak — `Config`, `Dispatch`, `Origin`, `Event`, `Hooks`, plus the compliance primitives `Secret`, `Pii`, `TimeSource`, `AuditLogger`, `CorrelationId`. |
| **Dispatch** | A callable entry point on an RTM. Implements `Dispatch<T: Config>` or `DispatchAsync<T: Config>`. Takes an `Origin`; returns `Result<_, DispatchError>`. |
| **Origin** | The caller identity carried with every dispatch. A type implementing `OriginTrait`; `BaseOrigin<P>` is the supplied default with variants `System` / `Service` / `User` / `Anonymous`. |
| **Sidecar** | An RTM (or runtime) running in a separate process behind an API adapter. Phase 2 adds `SidecarServer<R>` and `SidecarClient<R>` transports; the RTM itself is unchanged. |
| **Audit chain** | The BLAKE3 hash chain produced by `FileAuditLogger`. Each entry's `prev_hash` equals the hash of the previous entry's canonical serialised form. |
| **Compliance primitive** | A type or trait in the kernel that exists specifically to satisfy a regulatory requirement (e.g. `Secret`, `Pii`, `AuditLogger`, `CorrelationId`, `TimeSource`). |
| **Compliance matrix** | `docs/compliance.md`. The hand-maintained mapping from regulation articles to primitives to tests. `cargo test -p compliance-matrix-test` verifies it stays in sync with on-disk test files. |
| **Call enum** | A typed enum whose variants enumerate an RTM's dispatchable operations. Optional in Phase 1 (typed methods on the RTM are sufficient); Phase 3's `compose_runtime!` will generate one per RTM for runtime-wide routing. |

## Acronyms

| Acronym | Meaning |
|---|---|
| **AFIT** | `async fn` in trait. Stable since Rust 1.75. Used by `DispatchAsync` and `Hooks`. |
| **BLAKE3** | The cryptographic hash function backing the audit-log chain. |
| **DORA** | EU Digital Operational Resilience Act (Regulation 2022/2554). |
| **eIDAS** | EU Electronic Identification, Authentication and Trust Services (Reg. 2024/1183 for 2.0). |
| **ENISA** | EU Agency for Cybersecurity. |
| **GDPR** | EU General Data Protection Regulation (Reg. 2016/679). |
| **NIS2** | EU Network and Information Systems Directive 2 (Reg. 2022/2555). |
| **PHC** | Password Hashing Competition string format used by Argon2 (`$argon2id$v=19$...`). |
| **PII** | Personally Identifiable Information. In this kit, tagged via `Pii<T, C>`. |
| **RTM** | Runtime Module (see Kit terms). |
| **W3C trace context** | The W3C-standardised `traceparent` HTTP header format used for distributed tracing. |
