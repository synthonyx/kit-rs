//! Parameter-source macros and const-generic getter types.
//!
//! Each macro in this crate generates a zero-sized struct that implements
//! [`synthonyx_kit_core::Get<T>`], so callers can pass values via the type
//! system rather than as runtime data.
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

/// Defines a struct that implements [`synthonyx_kit_core::Get<T>`] and returns
/// the provided expression on `get()`.
///
/// The expression is converted to the declared type via `Into`, so any value
/// satisfying `Into<T>` is accepted.
///
/// # Example
///
/// ```
/// use synthonyx_kit_primitives::param;
/// use synthonyx_kit_core::Get;
///
/// param!(Greeting: String = "Hello, World!");
/// assert_eq!(Greeting::get(), "Hello, World!");
/// ```
#[macro_export]
macro_rules! param {
    ($name:ident: $type:ty = $value:expr) => {
        #[doc = concat!("Type-level parameter (`", stringify!($type), "`).")]
        pub struct $name;
        impl ::synthonyx_kit_core::Get<$type> for $name {
            fn get() -> $type {
                $value.into()
            }
        }
    };
}

/// Defines a struct that implements [`synthonyx_kit_core::Get<String>`] and
/// reads its value from an environment variable on each `get()`.
///
/// Two forms are supported:
/// - `env_param!(Name = "VAR")` — panics if `VAR` is unset.
/// - `env_param!(Name = "VAR" or "default")` — returns the literal default if
///   `VAR` is unset.
///
/// # Example
///
/// ```
/// use synthonyx_kit_primitives::env_param;
/// use synthonyx_kit_core::Get;
///
/// // SAFETY: rust 2024 requires `unsafe` for env mutation; safe in a single-threaded doctest.
/// unsafe { std::env::set_var("MY_ENV_VAR", "Hello, World!"); }
///
/// env_param!(MyEnvParam = "MY_ENV_VAR");
/// assert_eq!(MyEnvParam::get(), "Hello, World!");
///
/// env_param!(WithDefault = "BOGUS_ENV_VAR" or "fallback");
/// assert_eq!(WithDefault::get(), "fallback");
/// ```
#[macro_export]
macro_rules! env_param {
    ($name:ident = $var:literal) => {
        #[doc = concat!("Environment-variable parameter reading `", $var, "`.")]
        pub struct $name;
        impl ::synthonyx_kit_core::Get<String> for $name {
            fn get() -> String {
                ::std::env::var($var).expect(concat!("env var ", $var, " must be set"))
            }
        }
    };
    ($name:ident = $var:literal or $or:expr) => {
        #[doc = concat!("Environment-variable parameter reading `", $var, "` (with default).")]
        pub struct $name;
        impl ::synthonyx_kit_core::Get<String> for $name {
            fn get() -> String {
                ::std::env::var($var).unwrap_or_else(|_| $or.into())
            }
        }
    };
}

macro_rules! const_getter {
    ($name:ident, $type:ty, $doc:literal) => {
        #[doc = $doc]
        pub struct $name<const T: $type>;
        impl<const T: $type> ::synthonyx_kit_core::Get<$type> for $name<T> {
            fn get() -> $type {
                T
            }
        }
    };
}

const_getter!(ConstI8, i8, "Type-level `i8` value.");
const_getter!(ConstI16, i16, "Type-level `i16` value.");
const_getter!(ConstI32, i32, "Type-level `i32` value.");
const_getter!(ConstI64, i64, "Type-level `i64` value.");
const_getter!(ConstI128, i128, "Type-level `i128` value.");
const_getter!(ConstIsize, isize, "Type-level `isize` value.");

const_getter!(ConstU8, u8, "Type-level `u8` value.");
const_getter!(ConstU16, u16, "Type-level `u16` value.");
const_getter!(ConstU32, u32, "Type-level `u32` value.");
const_getter!(ConstU64, u64, "Type-level `u64` value.");
const_getter!(ConstU128, u128, "Type-level `u128` value.");
const_getter!(ConstUsize, usize, "Type-level `usize` value.");

const_getter!(ConstBool, bool, "Type-level `bool` value.");

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use synthonyx_kit_core::Get;

    #[test]
    fn const_getters_work() {
        assert_eq!(ConstI8::<42>::get(), 42);
        assert_eq!(ConstI16::<42>::get(), 42);
        assert_eq!(ConstI32::<42>::get(), 42);
        assert_eq!(ConstI64::<42>::get(), 42);
        assert_eq!(ConstI128::<42>::get(), 42);
        assert_eq!(ConstIsize::<42>::get(), 42);

        assert_eq!(ConstU8::<42>::get(), 42);
        assert_eq!(ConstU16::<42>::get(), 42);
        assert_eq!(ConstU32::<42>::get(), 42);
        assert_eq!(ConstU64::<42>::get(), 42);
        assert_eq!(ConstU128::<42>::get(), 42);
        assert_eq!(ConstUsize::<42>::get(), 42);

        assert!(ConstBool::<true>::get());
        assert!(!ConstBool::<false>::get());
    }

    #[test]
    fn param_getter_works() {
        param!(S: String = "Hello");
        assert_eq!(S::get(), String::from("Hello"));
    }

    #[test]
    #[allow(
        unsafe_code,
        clippy::disallowed_methods,
        reason = "rust 2024 requires `unsafe` for env mutation; test-scoped variable"
    )]
    fn env_param_getter_works() {
        // SAFETY: this test runs within its own scope and mutates a
        // test-scoped variable. No other thread reads it.
        unsafe {
            std::env::set_var("SYNTHONYX_KIT_PRIMITIVES_TEST_VAR", "set-value");
        }
        env_param!(Q = "SYNTHONYX_KIT_PRIMITIVES_TEST_VAR");
        assert_eq!(Q::get(), "set-value");

        env_param!(X = "SYNTHONYX_KIT_PRIMITIVES_TEST_UNSET_VAR" or "fallback");
        assert_eq!(X::get(), "fallback");
    }
}
