//! The `Get<T>` trait — values supplied as types.

/// A trait to obtain a single value from a type, statelessly.
///
/// `Get<T>` lets callers pass values via the type system rather than as
/// runtime data. The blanket impl for `()` gives `T::default()`, so `()` is
/// the conventional "use the default" parameter throughout the kit.
pub trait Get<I> {
    /// Returns the value carried by this type.
    fn get() -> I;
}

impl<T> Get<T> for ()
where
    T: Default,
{
    fn get() -> T {
        T::default()
    }
}
