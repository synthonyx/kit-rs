//! Password handling for the Synthonyx Kit.
//!
//! Defines the [`PasswordChecker`] trait and [`PasswordError`], plus the
//! [`Argon2Password`] reference implementation (Argon2id v19,
//! ENISA-compliant defaults).
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

mod argon2id;
pub use argon2id::Argon2Password;

use thiserror::Error;

/// Password-related errors.
#[derive(Clone, Debug, Error)]
pub enum PasswordError {
    /// Hashing failed (salt generation, parameter error, etc.).
    #[error("failed to hash password: {0}")]
    Hashing(String),

    /// Verification failed (malformed stored hash, parse error, etc.).
    #[error("failed to verify password: {0}")]
    Verification(String),
}

/// A type that can verify a plaintext password against stored material.
pub trait PasswordChecker {
    /// The plaintext password type. Typically `Secret<String>` so the
    /// plaintext zeroizes after verification.
    type Password;

    /// Verify `password`. Returns `Ok(true)` on match, `Ok(false)` on
    /// mismatch, and `Err` only on hashing/parsing failure.
    fn verify(&self, password: Self::Password) -> Result<bool, PasswordError>;
}
