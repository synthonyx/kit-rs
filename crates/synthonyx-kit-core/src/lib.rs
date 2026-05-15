//! Kernel traits for the Synthonyx Kit.
//!
//! This crate defines the contract every Runtime Module (RTM) speaks. It is
//! kept intentionally small and dependency-light so it can be depended on by
//! every other kit crate without circular dependencies.
//!
//! ## Module map
//!
//! - [`get`]: the `Get<T>` type-parameter pattern.
//! - [`config`]: the per-RTM `Config` trait that wires events, errors, origin,
//!   time, and audit.
//! - [`dispatch`]: synchronous and asynchronous dispatch entry points.
//! - [`origin`]: `OriginTrait`, `OriginKind`, and the generic `BaseOrigin<P>`.
//! - [`event`]: the `Event` trait and `Severity` tag.
//! - [`hooks`]: service-lifecycle hooks (`on_boot`, `on_shutdown`, etc.).
//! - [`error`]: the kit-wide `DispatchError`.
//! - [`correlation`]: trace and span identifiers.
//! - [`time`]: `TimeSource`, `SystemClock`, `MockClock`.
//! - [`secret`]: `Secret<T>` — redacted, zeroize-on-drop wrapper.
//! - [`pii`]: `Pii<T, C>` — type-tagged personal data.
//! - [`audit`]: the `AuditLogger` trait and `AuditEntry` type. Concrete sinks
//!   live in `synthonyx-kit-audit`.
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

pub mod audit;
pub mod config;
pub mod correlation;
pub mod dispatch;
pub mod error;
pub mod event;
pub mod get;
pub mod hooks;
pub mod origin;
pub mod pii;
pub mod secret;
pub mod time;

pub use audit::{AuditEntry, AuditError, AuditLogger, AuditValue, OriginSnapshot, Outcome};
pub use config::Config;
pub use correlation::{CorrelationId, SpanId};
pub use dispatch::{Dispatch, DispatchAsync, DispatchAsyncSend};
pub use error::DispatchError;
pub use event::{Event, EventBus, Severity};
pub use get::Get;
pub use hooks::Hooks;
pub use origin::{BaseOrigin, OriginKind, OriginTrait};
pub use pii::{Personal, Pii, PiiCategory, Pseudonymous, Sensitive};
pub use secret::Secret;
pub use time::{MockClock, MonotonicNanos, SystemClock, TimeSource, UnixNanos};
