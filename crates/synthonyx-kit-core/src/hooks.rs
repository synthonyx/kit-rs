//! Service-lifecycle hooks.

use crate::error::DispatchError;
use crate::time::MonotonicNanos;

/// Lifecycle hooks invoked by the runtime composer.
///
/// RTMs override individual methods as needed; the rest default to no-ops.
/// Calls are issued in declaration order on boot/migrate/idle and in reverse
/// order on shutdown (LIFO) so resources stack and unstack consistently.
pub trait Hooks {
    /// Called once at service boot, after `Config` is wired and storage is
    /// available.
    fn on_boot(&self) -> impl std::future::Future<Output = Result<(), DispatchError>> + Send {
        async { Ok(()) }
    }

    /// Called once during graceful shutdown.
    fn on_shutdown(&self) -> impl std::future::Future<Output = Result<(), DispatchError>> + Send {
        async { Ok(()) }
    }

    /// Schema/data migration entry point. Called by the migration
    /// orchestrator, not on every boot.
    fn on_migrate(
        &self,
        from: u32,
        to: u32,
    ) -> impl std::future::Future<Output = Result<(), DispatchError>> + Send {
        let _ = (from, to);
        async { Ok(()) }
    }

    /// Periodic idle work (e.g. retention sweeps). The deadline is a soft
    /// hint; implementations should return promptly when exceeded.
    fn on_idle(
        &self,
        deadline: MonotonicNanos,
    ) -> impl std::future::Future<Output = Result<(), DispatchError>> + Send {
        let _ = deadline;
        async { Ok(()) }
    }
}
