/// A trait to obtain a single value from type.
pub trait Get<I> {
    fn get() -> I;
}

// Implement Get for anything that implements Default.
impl<T> Get<T> for ()
where
    T: Default,
{
    fn get() -> T {
        T::default()
    }
}
