//! The kit-wide dispatch error type.

/// The kit-wide error returned by [`crate::Dispatch::call`] and
/// [`crate::DispatchAsync::call`].
///
/// Per-RTM concrete errors compose into this enum via `Into<DispatchError>`,
/// typically by wrapping a `thiserror::Error`-derived module error in
/// [`DispatchError::module`].
#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    /// A module-specific error.
    #[error("module `{module}` failed: {source}")]
    Module {
        /// The module identifier, taken from [`crate::Config::MODULE`].
        module: &'static str,
        /// The underlying typed error.
        #[source]
        source: Box<dyn core::error::Error + Send + Sync>,
    },

    /// The origin was not permitted to perform this action.
    #[error("origin not permitted for `{action}`")]
    BadOrigin {
        /// A short identifier for the action that was rejected.
        action: &'static str,
    },

    /// A precondition (input validation, invariant) was not satisfied.
    #[error("precondition failed: {0}")]
    Precondition(&'static str),

    /// An unexpected internal condition that should not occur in correct code.
    #[error("internal error: {0}")]
    Internal(String),
}

impl DispatchError {
    /// Construct a [`Self::Module`] variant from a module identifier and a
    /// typed error.
    pub fn module<E>(module: &'static str, source: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Module {
            module,
            source: Box::new(source),
        }
    }
}
