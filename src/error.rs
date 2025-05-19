#[derive(Debug)]
pub enum Error {
    DispatchError{
        module: &'static str,
        error: Box<dyn std::error::Error>
    },
    Other(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DispatchError{ module, error } => write!(f, "Module '{}' failed to dispatch with error: {}", module, error),
            Self::Other(error) => write!(f, "{}", error)
        }
    }
}

impl std::error::Error for Error {}