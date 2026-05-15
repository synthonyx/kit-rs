//! Domain event emission.

/// Severity tag attached to every emitted event.
///
/// Used by audit sinks and (in Phase 2) DORA incident-reporting.
/// `Security`/`Critical` are the hooks that escalate to mandatory reporting.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Routine informational event.
    Info,
    /// Notable but non-actionable.
    Notice,
    /// Recoverable problem; investigate.
    Warning,
    /// Security-relevant; potentially in scope for DORA incident reporting.
    Security,
    /// Critical/severe; triggers DORA Article 19 reporting in Phase 2.
    Critical,
}

/// A domain event emitted by an RTM.
///
/// Event emission is synchronous and infallible from the dispatch path; the
/// implementing bus is responsible for persistence and backpressure.
pub trait Event: Send + Sync + 'static {
    /// The originating module identifier (see [`crate::Config::MODULE`]).
    fn module(&self) -> &'static str;
    /// A stable name for this event variant.
    fn name(&self) -> &'static str;
    /// The severity tag.
    fn severity(&self) -> Severity;
}

/// A sink that receives emitted events.
pub trait EventBus<E: Event>: Send + Sync + 'static {
    /// Publish an event. Must not block the caller.
    fn emit(&self, event: E);
}
