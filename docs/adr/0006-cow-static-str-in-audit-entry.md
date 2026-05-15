# 0006 — `Cow<'static, str>` for `AuditEntry::module` and `action`

**Status:** Accepted

## Context

`AuditEntry::module` and `action` are almost always string literals at
construction time — `Config::MODULE` is a `&'static str`, and the action
name is typically a literal like `"register_user"`. The initial design
declared both fields as `&'static str`.

This works perfectly for construction but breaks deserialisation:
`serde_json::from_str::<AuditEntry>("...")` cannot produce `&'static str`
references — the data is owned by the deserialiser's input buffer, not
static memory. Since `FileAuditLogger` reads entries back via
`verify_chain`, this is a hard blocker.

Alternatives considered:

- **`String`** for both fields. Works for deserialisation; requires a
  heap allocation per construction even when a literal would suffice.
- **`&'a str` with a lifetime parameter on `AuditEntry`**. Avoids allocation
  but propagates the lifetime through every API that touches an entry,
  including the `AuditLogger` trait. Heavy ergonomic cost.
- **A separate `OwnedAuditEntry` for round-tripping**. Two types, with
  conversion impls. Doubles the surface; users must remember which one to
  use where.
- **`Cow<'static, str>`**. `Cow::Borrowed("literal")` at construction (zero
  cost); `Cow::Owned(String)` after deserialisation.

## Decision

`AuditEntry::module: Cow<'static, str>` and
`AuditEntry::action: Cow<'static, str>`. `OriginSnapshot::service` follows
the same pattern.

Callers construct entries as:
```rust
AuditEntry {
    module: Cow::Borrowed(<T as Config>::MODULE),
    action: Cow::Borrowed("register_user"),
    // ...
}
```

## Consequences

**Positive.** Zero-cost construction from literals. Deserialisation
produces `Cow::Owned(String)` automatically. Single type covers both
flows.

**Negative.** Slightly noisier construction syntax than bare `&'static str`.
We considered macro sugar but decided the explicit `Cow::Borrowed(...)`
is preferable — it documents the intent and is the right hint when
forgetting to wrap (the error message points directly at it).

**Forward path.** Phase 3's `compose_runtime!` will emit
`Cow::Borrowed(...)` in the generated code, so RTM authors using the
macro never type it by hand.
