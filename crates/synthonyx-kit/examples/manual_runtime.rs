//! Manual runtime composition demonstration.
//!
//! Mirrors the walkthrough in `docs/writing-an-rtm.md`. Demonstrates the
//! minimum surface needed to write an RTM and wire it into a concrete
//! `Runtime` type without the Phase 3 `compose_runtime!` macro.
//!
//! Run with `cargo run --example manual_runtime -p synthonyx-kit`.

use std::borrow::Cow;
use std::collections::BTreeMap;

use synthonyx_kit::audit::TracingAuditLogger;
use synthonyx_kit::core::{
    AuditEntry, AuditLogger, BaseOrigin, Config, CorrelationId, DispatchError, Event,
    OriginSnapshot, OriginTrait, Outcome, Severity, SystemClock, TimeSource,
};

// ---------------------------------------------------------------------------
// 1. The RTM's Event
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum UsersEvent {
    UserRegistered { name: String },
}

impl Event for UsersEvent {
    fn module(&self) -> &'static str {
        "users"
    }
    fn name(&self) -> &'static str {
        match self {
            UsersEvent::UserRegistered { .. } => "user_registered",
        }
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
}

// ---------------------------------------------------------------------------
// 2. The RTM's Error
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 3. The RTM's Config extension trait
// ---------------------------------------------------------------------------

pub trait UsersConfig: Config<Event = UsersEvent, Error = UsersError> {}

// ---------------------------------------------------------------------------
// 4. The RTM itself — a stateful struct
// ---------------------------------------------------------------------------

pub struct UsersRtm<T: UsersConfig> {
    audit: T::Audit,
    time: T::Time,
}

impl<T: UsersConfig> UsersRtm<T> {
    pub fn new(audit: T::Audit, time: T::Time) -> Self {
        Self { audit, time }
    }

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

// ---------------------------------------------------------------------------
// 5. The runtime composer — hand-written today, macro-generated in Phase 3
// ---------------------------------------------------------------------------

struct MyRuntime;

impl Config for MyRuntime {
    const MODULE: &'static str = "users";
    type RuntimeEvent = UsersEvent;
    type Event = UsersEvent;
    type Error = UsersError;
    type Origin = BaseOrigin<String>;
    type Time = SystemClock;
    type Audit = TracingAuditLogger;
}

impl UsersConfig for MyRuntime {}

// ---------------------------------------------------------------------------
// 6. Wiring and calling
// ---------------------------------------------------------------------------

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

    // Demonstrate the EmptyName error path:
    let origin = BaseOrigin::User {
        principal: "user-42".to_string(),
        correlation: CorrelationId([42u8; 16]),
    };
    let rejected = users.register_user(origin, "".to_string());
    assert!(matches!(rejected, Err(DispatchError::Module { .. })));
    println!("Empty name rejected as expected.");
}
