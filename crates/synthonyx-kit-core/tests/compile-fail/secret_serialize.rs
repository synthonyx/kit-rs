//! `Secret<T>` must not implicitly serialise via `serde_json::to_string`.
//! Even with the kit's `serde` feature enabled, callers must opt in to
//! serialisation explicitly via `#[serde(with = "...")]`.

fn main() {
    let s = synthonyx_kit_core::Secret::new(String::from("x"));
    let _ = serde_json::to_string(&s);
}
