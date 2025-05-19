#[derive(Debug)]
pub enum Error {
    DispatchError{
        module: &'static str,
        error: Box<dyn std::error::Error>
    },
    Other(String)
}