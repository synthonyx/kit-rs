/// A crate for Argon2 based passwords.
use std::sync::{Arc, Mutex};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use crate::traits::password::{PasswordChecker, PasswordError};

/// Hashes a given password using the default settings for Argon2id (v19).
///
/// # Errors
///
/// Returns an error if there's an issue generating the salt or hashing the password.
fn hash_password(password: String) -> Result<String, argon2::password_hash::Error> {
    // Generate a random salt to be used with the password.
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
}

/// Verifies a given password against a hashed password.
///
/// # Errors
///
/// Returns an error if there's an issue parsing the hash or verifying the password.
fn verify_password(
    password: String,
    hash: String
) -> Result<bool, argon2::password_hash::Error> {
    // Parse the provided hash to extract its contents.
    let parsed_hash = PasswordHash::new(&hash)?;

    // Verify the password using Argon2's default settings.
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Implements conversion from `argon2::password_hash::Error` to our custom `PasswordError`.
impl From<argon2::password_hash::Error> for PasswordError {
    /// Converts an Argon2 error into a `PasswordError`.
    fn from(err: argon2::password_hash::Error) -> Self {
        PasswordError::Hashing(err.to_string())
    }
}

/// A password implementation using Argon2.
///
/// This struct stores the hashed password in an `Arc`-protected `Mutex`, allowing for thread-safe access and modification
/// and implements the PasswordChecker and PasswordHasher traits.
#[derive(Clone, Debug)]
pub struct Argon2Password(Arc<Mutex<String>>);

impl PasswordChecker for Argon2Password {
    /// The type of passwords we're working with (in this case, `String`s).
    type Password = String;

    /// Verifies a given password against the stored hash.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue parsing the stored hash or verifying the password itself.
    fn verify(&self, password: Self::Password) -> Result<bool, PasswordError> {
        let password_hash = self.0.lock().map_err(|e| PasswordError::Other(e.to_string()))?;
        Ok(verify_password(password, password_hash.clone())?)
    }
}

impl Argon2Password {
    pub fn new(password: impl Into<String>) -> Result<Self, PasswordError> {
        Ok(Argon2Password(Arc::new(Mutex::<String>::new(hash_password(password.into())?))))
    }

    /// Returns the inner password hash as a `String`.
    pub fn to_inner(&self) -> String {
        self.0.lock().unwrap().clone()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Argon2Password {
    /// Serializes the stored hash to a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.lock().unwrap().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Argon2Password {
    /// Deserializes a stored hash from a string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Argon2Password(Arc::new(Mutex::new(s))))
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_password() {
        let password = "mysecretpassword".to_string();
        let hash = hash_password(password.clone()).unwrap();

        assert_eq!(hash.len(), 97);

        let argon2_password = Argon2Password(Arc::new(Mutex::new(hash)));
        assert!(argon2_password.verify(password).is_ok());
        assert!(!argon2_password.verify("wrongpassword".into()).unwrap());
    }

    #[test]
    fn test_serialization() {
        let password = "mysecretpassword".to_string();
        let argon2_password = Argon2Password::new(password.clone()).expect("Expected to be able to crate a new hashed password");
        let serialized = serde_json::to_string(&argon2_password).unwrap();

        let deserialized: Argon2Password = serde_json::from_str(&serialized).unwrap();
        assert_eq!(argon2_password.to_inner(), deserialized.to_inner());
    }
}