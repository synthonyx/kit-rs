# synthonyx-kit-password

Password handling for the Synthonyx Kit. Defines the `PasswordChecker`
trait and `PasswordError`; ships an `Argon2Password` reference
implementation using Argon2id v19 with the library's ENISA-compliant
defaults.

## Contents

- `PasswordChecker` — trait every password-hashing implementation
  satisfies. `type Password = Secret<String>` so the plaintext zeroizes
  after verification.
- `PasswordError` — `Hashing` / `Verification` variants. No panics.
- `Argon2Password` — Argon2id v19 backed by `Arc<str>` storing the PHC
  hash. Cheap to clone; immutable after construction; never logs the
  hash in `Debug` output.

```rust
use synthonyx_kit_core::Secret;
use synthonyx_kit_password::{Argon2Password, PasswordChecker};

let p = Argon2Password::new(Secret::new("hunter2".to_string()))?;
assert!(p.verify(Secret::new("hunter2".to_string()))?);
assert!(!p.verify(Secret::new("wrong".to_string()))?);
# Ok::<(), synthonyx_kit_password::PasswordError>(())
```

With the `serde` feature, `Argon2Password` round-trips as the PHC-encoded
hash string — suitable for storing in a password database.
