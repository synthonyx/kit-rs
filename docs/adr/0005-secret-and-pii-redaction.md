# 0005 — `Secret<T>` and `Pii<T, C>` redaction patterns

**Status:** Accepted

## Context

GDPR Art. 32, ISO/IEC 27002 control A.5.34, and ENISA cryptographic
guidance all converge on the same principle: sensitive material must not
leak via accidental logging, serialisation, or memory residue. Rust has no
type-level "do not log" tag out of the box.

Two distinct concerns:

- **Secrets**: passwords, API keys, session tokens. Should never be
  serialised in any form; should be zeroed when no longer needed.
- **Personal data (PII)**: user names, addresses, identifiers. Must
  *round-trip* through storage (the whole point is to persist them) but
  must not appear in logs or be mishandled in transit.

The two warrant different abstractions.

## Decision

### `Secret<T: Zeroize>`

- Wraps `T` (any `zeroize::Zeroize` type, including `String` and `Vec<u8>`).
- `Debug` prints `Secret(<redacted>)` — no payload.
- **Does not implement `Display`**, period. Enforced via
  `static_assertions::assert_not_impl_any!`.
- **Does not implement `Serialize` / `Deserialize`**, even with the kit's
  `serde` feature enabled. Callers must opt in via `#[serde(with = "...")]`
  on a per-field basis. Enforced via static_assertions + trybuild compile-
  fail tests.
- `Drop` calls `Zeroize::zeroize` on the inner value.
- `expose()` is the single, explicitly-named way to read the inner value;
  every call site is therefore reviewable.

### `Pii<T, C: PiiCategory>`

- Wraps `T` with a zero-sized type tag `C` (`Personal` / `Sensitive` /
  `Pseudonymous`).
- `Debug` prints `Pii<category>(<redacted>)` — the category is visible
  (auditors need to see *what kind* of data they are looking at), the
  payload is not.
- **Does not implement `Display`**, enforced via `assert_not_impl_any`.
- *Does* implement `Serialize` / `Deserialize` transparently (the value
  round-trips as if unwrapped). Storage backends (Phase 2) dispatch on
  `C::CATEGORY` at write time to drive column-level encryption and at
  erasure time to route GDPR Art. 17 requests.

## Consequences

**Positive.** Strong compile-time guarantees, not runtime checks.
trybuild + static_assertions tests make the negative impls a permanent
part of the contract. Auditors can grep for `Secret<` / `Pii<` and trust
that the redaction is real.

**Negative.** Slightly more friction: callers must wrap explicitly and
unwrap via `.expose()`. We consider this friction a feature — it forces
every call site to be deliberate.

**Trade-off.** `Pii` does not zeroize. It round-trips through storage,
which is the opposite of a zeroize lifecycle. If a workflow needs zeroize
on PII (e.g., for in-memory pseudonymisation), compose `Secret<Pii<T, C>>`.

## Related

- [`docs/compliance.md`](../compliance.md) lists which tests enforce these
  contracts.
- [`crates/synthonyx-kit-core/tests/compliance_secret.rs`](../../crates/synthonyx-kit-core/tests/compliance_secret.rs)
  and `compliance_pii.rs` are the load-bearing tests.
