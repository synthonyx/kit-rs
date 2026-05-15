//! `Pii<T, C>` deliberately does not implement `Display` so values cannot
//! be interpolated into formatted strings. Debug exists (redacted); Display
//! does not.

fn main() {
    let p =
        synthonyx_kit_core::Pii::<String, synthonyx_kit_core::Personal>::new(String::from("x"));
    let _ = format!("{}", p);
}
