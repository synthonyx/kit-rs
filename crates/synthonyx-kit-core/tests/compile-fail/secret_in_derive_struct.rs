//! A struct that contains a `Secret<_>` field cannot auto-derive
//! `Serialize` without an explicit `#[serde(with = "...")]` attribute on
//! the secret field. This compile-fail test proves the implicit derive is
//! rejected.

#[derive(serde::Serialize)]
struct Wrapper {
    secret: synthonyx_kit_core::Secret<String>,
}

fn main() {
    let _ = Wrapper {
        secret: synthonyx_kit_core::Secret::new(String::new()),
    };
}
