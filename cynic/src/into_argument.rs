use crate::{argument::SerializableArgument, Id};

/// IntoArgument is used to type-check arguments to queries in cynic.
///
/// A GraphQL argument that accepts `String!` will accept any type that is
/// `IntoArgument<String>`.  Similarly, an optional `String` in GraphQL will
/// accept any `IntoArgument<Option<String>>`.
///
/// There are implementations of this for most of the built in scalars to allow
/// users to express arguments in a simple manner.  The `cynic::Enum` derive
/// also generates impls for converting options & refs easily.
pub trait IntoArgument<Argument>
where
    Argument: SerializableArgument + Send + 'static,
{
    fn into_argument(self) -> Argument;
}

impl<T> IntoArgument<T> for T
where
    T: SerializableArgument + Send + 'static,
{
    fn into_argument(self) -> T {
        self
    }
}

/// Defines useful argument conversions for scalar-like types
///
/// Mostly just converts references to owned via cloning and
/// non option-wrapped types into Option where appropriate.
#[macro_export]
macro_rules! define_into_argument_for_scalar {
    ($inner:ty) => {
        impl $crate::IntoArgument<Option<$inner>> for $inner {
            fn into_argument(self) -> Option<$inner> {
                Some(self)
            }
        }

        impl $crate::IntoArgument<Option<$inner>> for &$inner {
            fn into_argument(self) -> Option<$inner> {
                Some(self.clone())
            }
        }
    };
}

macro_rules! define_into_argument_for_option_refs {
    ($inner:ty) => {
        impl $crate::IntoArgument<Option<$inner>> for Option<&$inner> {
            fn into_argument(self) -> Option<$inner> {
                self.cloned()
            }
        }

        impl $crate::IntoArgument<Option<$inner>> for &Option<$inner> {
            fn into_argument(self) -> Option<$inner> {
                self.clone()
            }
        }
    };
}

define_into_argument_for_scalar!(i32);
define_into_argument_for_scalar!(f64);
define_into_argument_for_scalar!(String);
define_into_argument_for_scalar!(bool);
define_into_argument_for_scalar!(Id);

define_into_argument_for_option_refs!(i32);
define_into_argument_for_option_refs!(f64);
define_into_argument_for_option_refs!(String);
define_into_argument_for_option_refs!(bool);
define_into_argument_for_option_refs!(Id);

impl IntoArgument<String> for &str {
    fn into_argument(self) -> String {
        self.to_string()
    }
}

impl IntoArgument<Option<String>> for &str {
    fn into_argument(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoArgument<Option<String>> for Option<&str> {
    fn into_argument(self) -> Option<String> {
        self.map(|s| s.to_string())
    }
}
