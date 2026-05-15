//! Dispatch traits — entry points for RTM operations.

use crate::config::Config;
use crate::error::DispatchError;

/// Synchronous dispatch entry point.
///
/// RTMs implement `Dispatch` on a typed "call" enum whose variants name the
/// dispatchable operations. The `compose_runtime!` macro (Phase 3) will
/// generate these impls; Phase 1/2 callers write them by hand.
pub trait Dispatch<T: Config> {
    /// The value produced on success.
    type Output;

    /// Execute this dispatch on behalf of `origin`.
    fn call(&self, origin: T::Origin) -> Result<Self::Output, DispatchError>;
}

/// Asynchronous dispatch entry point.
///
/// Uses `async fn` in trait (stable since 1.75). `trait_variant::make`
/// generates a `DispatchAsyncSend` variant with `Send`-bounded futures for
/// callers that need to spawn dispatches on a multi-threaded executor.
#[trait_variant::make(DispatchAsyncSend: Send)]
pub trait DispatchAsync<T: Config> {
    /// The value produced on success.
    type Output;

    /// Execute this dispatch on behalf of `origin`.
    async fn call(&self, origin: T::Origin) -> Result<Self::Output, DispatchError>;
}
