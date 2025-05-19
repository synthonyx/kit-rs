/// Defines a struct that implements the `Get<T>` trait which returns the
/// provided value. This is paticularly convenient in combination with associated
/// types of traits in cases that a value has to be stored into an associated type.
///
/// The `param` macro creates a new public struct with the specified name.
/// It then implements the `Get<T>` trait for this struct, providing a `get` method
/// that returns a value of type `T`, converted from the provided expression.
///
/// # Arguments
///
/// * `$name` - The name of the struct to be created.
/// * `$type` - The type of the value to be returned by the `get` method.
/// * `$value` - The expression that provides the value to be returned. This expression
///   will be converted to the specified type using the `into` method.
///
/// # Example
///
/// ```
/// use synthonyx_kit::param;
/// use synthonyx_kit::traits::get::Get;
///
/// // Define a struct `MyStringParam` that implements `Get<String>` and returns the value "Hello, World!"
/// param!(MyStringParam, String, "Hello, World!");
///
/// fn main() {
///     let value_string = MyStringParam::get();
///     println!("Value (String): {}", value_string); // Output: Value (String): Hello, World!
/// }
/// ```
///
/// # Notes
///
/// - The `$value` expression should be convertible to the specified `$type` using the `into` method.
/// - The macro generates a public struct, so the struct and its `get` method are accessible outside the module.
#[macro_export]
macro_rules! param {
    ($name:ident, $type:ty, $value:expr) => {
        pub struct $name;
        impl $crate::traits::get::Get<$type> for $name {
            fn get() -> $type {
                $value.into()
            }
        }
    };
}

/// Defines a struct that implements the `Get<T>` trait which returns the
/// value of an environment variable.
///
/// The `env_param` macro creates a new public struct with the specified name.
/// It then implements the `Get<T>` trait for this struct, providing a `get` method
/// that returns a value of type `T`, converted from the provided expression.
///
/// # Arguments
///
/// * `$name` - The name of the struct to be created.
/// * `$type` - The type of the value to be returned by the `get` method.
/// * `$var` - The name of the environment variable. This value will be
///   converted to the specified type using the `into` method.
///
/// # Example
///
/// ```
/// use synthonyx_kit::env_param;
/// use synthonyx_kit::traits::get::Get;
///
/// env_param!(MyEnvParam, String, "MY_ENV_VAR");
///
/// fn main() {
///     // Set an environment variable to make sure the demo won't fail.
///     std::env::set_var("MY_ENV_VAR", "Hello, World!");
///
///     let env_value = MyEnvParam::get();
///     println!("Value (String): {}", env_value); // Output: Value (String): Hello, World!
/// }
/// ```
///
/// # Notes
///
/// - The value of the environment vairable should be convertible to the specified `$type` using the `into` method.
/// - The macro generates a public struct, so the struct and its `get` method are accessible outside the module.
#[macro_export]
macro_rules! env_param {
    ($name:ident, $type:ty, $var:expr) => {
        pub struct $name;
        impl $crate::traits::get::Get<$type> for $name {
            fn get() -> $type {
                // Load environment variables from .env file
                dotenv::dotenv().ok();

                // Get the API key from the environment variable
                let res = std::env::var($var).expect("$var must be set");
                res.into()
            }
        }
    };
}

// A macro to define a constant type of Rust primitives.
macro_rules! const_getter {
    ($name:ident, $type:ty) => {
        pub struct $name<const T: $type>;
        impl<const T: $type> $crate::traits::get::Get<$type> for $name<T> {
            fn get() -> $type {
                T
            }
        }
    };
}

const_getter!(ConstI8, i8);
const_getter!(ConstI16, i16);
const_getter!(ConstI32, i32);
const_getter!(ConstI64, i64);
const_getter!(ConstI128, i128);
const_getter!(ConstIsize, isize);

const_getter!(ConstU8, u8);
const_getter!(ConstU16, u16);
const_getter!(ConstU32, u32);
const_getter!(ConstU64, u64);
const_getter!(ConstU128, u128);
const_getter!(ConstUsize, usize);

const_getter!(ConstBool, bool);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::get::Get;

    #[test]
    fn const_getters_works() {
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

        assert_eq!(ConstBool::<true>::get(), true);
    }

    #[test]
    fn param_getter_works() {
        param!(S, String, "Hello");
        assert_eq!(S::get(), "Hello".to_string());
    }

    #[test]
    fn env_param_getter_works() {
        // Set an environment variable to make sure the demo won't fail.
        unsafe {
            std::env::set_var("MY_ENV_VAR", "Hello, World!");
        }
        env_param!(Q, String, "MY_ENV_VAR");
        assert_eq!(Q::get(), "Hello, World!".to_string());
    }
}
