//! Argon2id password hashing reference implementation.
//!
//! ENISA-compliant defaults: Argon2id v19, library defaults for memory cost,
//! time cost, and parallelism. Salt is a 16-byte random value drawn from
//! `OsRng` and persisted in the PHC-encoded output.

use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use synthonyx_kit_core::Secret;

use crate::{PasswordChecker, PasswordError};

fn hash_password(plaintext: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(plaintext.as_bytes(), &salt)?
        .to_string())
}

fn verify_password(plaintext: &str, phc_hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(phc_hash)?;
    Ok(Argon2::default()
        .verify_password(plaintext.as_bytes(), &parsed)
        .is_ok())
}

impl From<argon2::password_hash::Error> for PasswordError {
    fn from(err: argon2::password_hash::Error) -> Self {
        PasswordError::Hashing(err.to_string())
    }
}

/// An Argon2id-hashed password.
///
/// Stores the PHC-encoded hash string in an `Arc<str>` — immutable after
/// construction, cheap to clone, no lock-poisoning surface. The plaintext is
/// always passed as [`Secret<String>`] so it zeroizes when dropped.
#[derive(Clone)]
pub struct Argon2Password(Arc<str>);

impl Argon2Password {
    /// Hash `plaintext` with Argon2id and return an [`Argon2Password`].
    ///
    /// The plaintext is zeroized when `plaintext` drops, which happens as
    /// soon as this function returns.
    pub fn new(plaintext: Secret<String>) -> Result<Self, PasswordError> {
        let hash = hash_password(plaintext.expose())?;
        Ok(Self(Arc::from(hash.into_boxed_str())))
    }

    /// Construct an [`Argon2Password`] from an existing PHC-encoded hash
    /// string. Use this when loading a hash from your password store.
    pub fn from_hash(phc_hash: impl Into<String>) -> Self {
        Self(Arc::from(phc_hash.into().into_boxed_str()))
    }

    /// Borrow the PHC-encoded hash string. Useful for persisting to a store.
    pub fn as_hash_str(&self) -> &str {
        &self.0
    }
}

impl PasswordChecker for Argon2Password {
    type Password = Secret<String>;

    fn verify(&self, password: Self::Password) -> Result<bool, PasswordError> {
        Ok(verify_password(password.expose(), &self.0)?)
    }
}

impl core::fmt::Debug for Argon2Password {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Argon2Password(<redacted>)")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Argon2Password {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Argon2Password {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Self::from_hash(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret(s: &str) -> Secret<String> {
        Secret::new(s.to_string())
    }

    #[test]
    fn round_trip_verify_succeeds_and_rejects_wrong_password() {
        let p = Argon2Password::new(secret("mysecretpassword")).unwrap();
        assert!(p.verify(secret("mysecretpassword")).unwrap());
        assert!(!p.verify(secret("wrongpassword")).unwrap());
    }

    #[test]
    fn debug_does_not_leak_hash() {
        let p = Argon2Password::new(secret("mysecretpassword")).unwrap();
        let d = format!("{p:?}");
        assert_eq!(d, "Argon2Password(<redacted>)");
        // PHC hashes start with "$argon2id$" — confirm it's not in Debug.
        assert!(!d.contains("$argon2"));
    }

    #[test]
    fn as_hash_str_returns_phc_format() {
        let p = Argon2Password::new(secret("mysecretpassword")).unwrap();
        assert!(p.as_hash_str().starts_with("$argon2id$"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_and_verification_still_works() {
        let p = Argon2Password::new(secret("mysecretpassword")).unwrap();
        let s = serde_json::to_string(&p).unwrap();
        let p2: Argon2Password = serde_json::from_str(&s).unwrap();
        assert_eq!(p.as_hash_str(), p2.as_hash_str());
        assert!(p2.verify(secret("mysecretpassword")).unwrap());
    }
}
