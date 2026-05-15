//! The Synthonyx Kit: a Rust development kit for building EU-compliance-grade
//! (web) services around a modular runtime architecture.
//!
//! This crate is a facade: it re-exports curated APIs of the workspace member
//! crates so downstream users have a single dependency to add. Each sub-crate
//! can also be depended on individually for fine-grained dependency control.
//!
//! See `docs/standards.md` for the EU regulatory frameworks this kit targets.
//!
//! # Example
//!
//! ```
//! use synthonyx_kit::{param, primitives::ConstU32};
//! use synthonyx_kit::core::Get;
//!
//! param!(MaxRetries: u32 = 5u32);
//! assert_eq!(MaxRetries::get(), 5);
//! assert_eq!(ConstU32::<42>::get(), 42);
//! ```
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

pub use synthonyx_kit_core as core;
pub use synthonyx_kit_core::{
    AuditEntry, AuditError, AuditLogger, BaseOrigin, Config, CorrelationId, Dispatch,
    DispatchAsync, DispatchError, Event, EventBus, Get, Hooks, MockClock, OriginKind, OriginTrait,
    Personal, Pii, PiiCategory, Pseudonymous, Secret, Sensitive, Severity, SpanId, SystemClock,
    TimeSource, UnixNanos,
};
pub use synthonyx_kit_primitives as primitives;
pub use synthonyx_kit_primitives::{env_param, param};

#[cfg(feature = "storage")]
pub use synthonyx_kit_storage as storage;

#[cfg(feature = "audit")]
pub use synthonyx_kit_audit as audit;

#[cfg(feature = "compliance")]
pub use synthonyx_kit_compliance as compliance;

#[cfg(feature = "tracing")]
pub use synthonyx_kit_tracing as tracing;

#[cfg(feature = "password")]
pub use synthonyx_kit_password as password;
