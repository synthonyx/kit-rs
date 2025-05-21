use thiserror::Error;

/// This module contains traits and types related to password handling that every password type 
/// should implement to be compliant with this kit's standard. It includes a trait for password 
/// hashing and verification, an error type for handling errors during these operations,
/// and a trait that combines both.

#[derive(Clone, Debug, Error)]
/// Error type used when encountering issues with passwords.
pub enum PasswordError {
    /// Error occurred while trying to hash the password.
    #[error("Failed to hash password: {0}")]
    Hashing(String),
    
    /// Error occurred while trying to verify the password.
    #[error("Failed to verify password: {0}")]
    Verification(String),
    
    /// Other unknown error occurred.
    #[error("{0}")]
    Other(String),
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