/// This module contains traits and types related to password handling that every password type 
/// should implement to be compliant with this kit's standard. It includes a trait for password 
/// hashing and verification, an error type for handling errors during these operations,
/// and a trait that combines both.

#[derive(Clone, Debug)]
/// Error type used when encountering issues with passwords.
pub enum PasswordError {
    /// Error occurred while trying to hash the password.
    Hashing(String),
    /// Error occurred while trying to verify the password.
    Verification(String),
    /// Other unknown error occurred.
    Other(String),
}

impl std::fmt::Display for PasswordError {
    /// Formats this `PasswordError` instance as a string.
    ///
    /// The `Display` implementation provides a human-readable representation
    /// of the error. Each variant is formatted differently:
    /// - `Hashing`: "Failed to hash password: <error>"
    /// - `Verification`: "Failed to verify password: <error>"
    /// - `Other`: simply "<error>"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::Hashing(e) => write!(f, "Failed to hash password: {}", e),
            PasswordError::Verification(e) => write!(f, "Failed to verify password: {}", e),
            PasswordError::Other(e) => write!(f, "{}", e),
        }
    }
}

/// Trait for types that can hash passwords.
pub trait PasswordHasher {
    /// Type of password this hasher is designed to handle.
    type Password;

    /// Hashes the provided `password`.
    ///
    /// Returns an error if hashing fails. The error will be a variant
    /// of `PasswordError` indicating why hashing failed.
    fn hash(&self, password: Self::Password) -> Result<(), PasswordError>;
}

/// Trait for types that can verify passwords.
pub trait PasswordChecker {
    /// Type of password this checker is designed to handle.
    type Password;

    /// Verifies the provided `password`.
    ///
    /// Returns `Ok(true)` if the password matches and `Ok(false)` otherwise.
    /// If verification fails, returns an error which will be a variant
    /// of `PasswordError` indicating why verification failed.
    fn verify(&self, password: Self::Password) -> Result<bool, PasswordError>;
}

/// Trait for types that can both hash and verify passwords.
pub trait PasswordHandler: PasswordHasher + PasswordChecker {} 