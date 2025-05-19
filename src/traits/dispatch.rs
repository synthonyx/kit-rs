pub trait Dispatch {
    type Output;

    fn call(&self) -> Result<Self::Output, crate::error::Error>;
}

pub trait DispatchAsync {
    type Output;

    fn call(&self) -> Result<impl std::future::Future<Output = Self::Output> + Send, crate::error::Error>;
}