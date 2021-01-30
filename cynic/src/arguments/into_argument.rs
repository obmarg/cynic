use super::SerializableArgument;
use crate::Id;

/// IntoArgument is used to type-check arguments to queries in cynic.
///
/// A GraphQL argument that accepts `String!` will accept any type that is
/// `IntoArgument<String>`.  Similarly, an optional `String` in GraphQL will
/// accept any `IntoArgument<Option<String>>`.
///
/// There are implementations of this for most of the built in scalars to allow
/// users to express arguments in a simple manner.  The `cynic::Enum` derive
/// also generates impls for converting options & refs easily.
pub trait IntoArgument<T> {
    type Output: SerializableArgument;

    fn into_argument(self) -> Self::Output;
}

impl<T> IntoArgument<T> for T
where
    T: SerializableArgument,
{
    type Output = T;

    fn into_argument(self) -> T {
        self
    }
}

impl<'a, T> IntoArgument<Option<T>> for Option<&'a T>
where
    T: SerializableArgument,
{
    type Output = Option<&'a T>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<'a, T> IntoArgument<Option<T>> for &'a Option<T>
where
    T: SerializableArgument,
{
    type Output = Option<&'a T>;

    fn into_argument(self) -> Self::Output {
        self.as_ref()
    }
}

/// Defines useful argument conversions for scalar-like types
///
/// Mostly just converts references to owned via cloning and
/// non option-wrapped types into Option where appropriate.
#[macro_export]
macro_rules! impl_into_argument_for_options {
    ($inner:ty) => {
        impl $crate::IntoArgument<Option<$inner>> for $inner {
            type Output = Option<$inner>;

            fn into_argument(self) -> Option<$inner> {
                Some(self)
            }
        }

        impl<'a> $crate::IntoArgument<Option<$inner>> for &'a $inner {
            type Output = Option<&'a $inner>;

            fn into_argument(self) -> Option<&'a $inner> {
                Some(self)
            }
        }
    };
}

impl_into_argument_for_options!(i32);
impl_into_argument_for_options!(f64);
impl_into_argument_for_options!(String);
impl_into_argument_for_options!(bool);
impl_into_argument_for_options!(Id);

impl<'a> IntoArgument<String> for &'a str {
    type Output = &'a str;

    fn into_argument(self) -> &'a str {
        self
    }
}

impl<'a> IntoArgument<Option<String>> for &'a str {
    type Output = Option<&'a str>;

    fn into_argument(self) -> Option<&'a str> {
        Some(self)
    }
}

impl<'a> IntoArgument<Option<String>> for Option<&'a str> {
    type Output = Option<&'a str>;

    fn into_argument(self) -> Option<&'a str> {
        self
    }
}
