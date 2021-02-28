use super::SerializableArgument;
use crate::Id;

// TODO: MAybe rename this file conversions or similar, and document it?

/// IntoArgument is used to type-check arguments to queries in cynic.
///
/// A GraphQL argument that accepts `String!` will accept any type that is
/// `IntoArgument<String>`.  Similarly, an optional `String` in GraphQL will
/// accept any `IntoArgument<Option<String>>`.
///
/// There are implementations of this for most of the built in scalars to allow
/// users to express arguments in a simple manner.  The `cynic::Enum` derive
/// also generates impls for converting options & refs easily.
pub trait IntoArgument<TypeLock> {
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

pub trait EnumArgument<TypeLock> {
    type Output: SerializableArgument;

    fn into_argument(self) -> Self::Output;
}

pub trait InputObjectArgument<TypeLock> {
    type Output: SerializableArgument;

    fn into_argument(self) -> Self::Output;
}

/// An argument for scalars accepted as arguments.  This is used to determine whether
/// a type is compatible with a given scalar when provided as an argument: either directly
/// as an field argument or inside an InputObject.
///
/// This trait has two type parameters:
///
/// - `ScalarTypeLock` should point to the marker trait of the underlying scalar in your
///   `query_dsl` module.  This isn't directly used, but allows your crate to sidestep the
///   orphan rule which would otherwise prevent you from implementing this type on Options,
///   Vectors etc.
/// - `ArgumentTypeLock` is a type lock for the given argument type - this should contian
///    the `ScalarTypeLock` but wrapped in `Options`, or `Vec`s as appropraite to mirror the
///    arguments list/nullability .
pub trait ScalarArgument<ScalarTypeLock, ArgumentTypeLock> {
    type Output: SerializableArgument;

    fn into_argument(self) -> Self::Output;
}

macro_rules! def_argument_generics {
    {$arg_type:ident, $trait:path} => {
        impl<T, TypeLock> $arg_type<TypeLock> for T
        where
            T: $trait,
        {
            type Output = Self;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        impl<'a, T, TypeLock> $arg_type<Option<TypeLock>> for Option<&'a T>
        where
            T: $trait
        {
            type Output = Self;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        impl<'a, T, TypeLock> $arg_type<Option<TypeLock>> for &'a Option<T>
        where
            T: $trait,
        {
            type Output = Option<&'a T>;

            fn into_argument(self) -> Self::Output {
                self.as_ref()
            }
        }
    }
}

def_argument_generics!(EnumArgument, crate::Enum<TypeLock>);
def_argument_generics!(InputObjectArgument, crate::InputObject<TypeLock>);
//def_argument_generics!(ScalarArgument, crate::Scalar<TypeLock>);

// TODO: temp until I bring Enum & InputObject in line as well...
macro_rules! def_scalar_argument_generics {
    {$arg_type:ident, $trait:path} => {
        impl<T, TypeLock> $arg_type<TypeLock, TypeLock> for T
        where
            T: $trait,
        {
            type Output = Self;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        impl<'a, T, TypeLock> $arg_type<TypeLock, Option<TypeLock>> for Option<&'a T>
        where
            T: $trait
        {
            type Output = Self;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        impl<'a, T, TypeLock> $arg_type<TypeLock, Option<TypeLock>> for &'a Option<T>
        where
            T: $trait,
        {
            type Output = Option<&'a T>;

            fn into_argument(self) -> Self::Output {
                self.as_ref()
            }
        }
    }
}

def_scalar_argument_generics!(ScalarArgument, crate::Scalar<TypeLock>);

impl<T, TypeLock> EnumArgument<Option<Vec<TypeLock>>> for Option<Vec<T>>
where
    T: crate::Enum<TypeLock>,
{
    type Output = Option<Vec<T>>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<T, TypeLock> EnumArgument<Vec<TypeLock>> for Vec<T>
where
    T: crate::Enum<TypeLock>,
{
    type Output = Vec<T>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<T, TypeLock> EnumArgument<Option<Vec<Option<TypeLock>>>> for Option<Vec<Option<T>>>
where
    T: crate::Enum<TypeLock>,
{
    type Output = Option<Vec<Option<T>>>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<T, TypeLock> InputObjectArgument<Option<Vec<TypeLock>>> for Option<Vec<T>>
where
    T: crate::InputObject<TypeLock>,
{
    type Output = Option<Vec<T>>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<T, TypeLock> InputObjectArgument<Vec<TypeLock>> for Vec<T>
where
    T: crate::InputObject<TypeLock>,
{
    type Output = Vec<T>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<T, TypeLock> InputObjectArgument<Option<Vec<Option<TypeLock>>>> for Option<Vec<Option<T>>>
where
    T: crate::InputObject<TypeLock>,
{
    type Output = Option<Vec<Option<T>>>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

/// Defines useful argument conversions for input objects
///
/// Mostly just converts references to owned via cloning and
/// non option-wrapped types into Option where appropriate.
#[macro_export]
macro_rules! impl_common_input_object_argument_conversions {
    ($inner:ty, $type_lock:path) => {
        impl $crate::InputObjectArgument<Option<$type_lock>> for $inner {
            type Output = Option<$inner>;

            fn into_argument(self) -> Option<$inner> {
                Some(self)
            }
        }

        impl<'a> $crate::InputObjectArgument<Option<$type_lock>> for &'a $inner {
            type Output = Option<&'a $inner>;

            fn into_argument(self) -> Option<&'a $inner> {
                Some(self)
            }
        }

        // TODO: Try and implement list coercion in here...
    };
}

impl<'a> ScalarArgument<String, String> for &'a str {
    type Output = &'a str;

    fn into_argument(self) -> &'a str {
        self
    }
}

impl<'a> ScalarArgument<String, Option<String>> for &'a str {
    type Output = Option<&'a str>;

    fn into_argument(self) -> Option<&'a str> {
        Some(self)
    }
}

impl<'a> ScalarArgument<String, Option<String>> for Option<&'a str> {
    type Output = Option<&'a str>;

    fn into_argument(self) -> Option<&'a str> {
        self
    }
}

/// Defines useful argument conversions for input objects
///
/// Mostly just converts references to owned via cloning and
/// non option-wrapped types into Option where appropriate.
#[macro_export]
macro_rules! impl_scalar_argument {
    ($type:ty, $type_lock:path) => {
        impl $crate::ScalarArgument<$type_lock, Option<$type_lock>> for $type {
            type Output = Option<$type>;

            fn into_argument(self) -> Option<$type> {
                Some(self)
            }
        }

        impl $crate::ScalarArgument<$type_lock, Option<$type_lock>> for Option<$type> {
            type Output = Option<$type>;

            fn into_argument(self) -> Option<$type> {
                self
            }
        }

        impl<'a> $crate::ScalarArgument<$type_lock, Option<$type_lock>> for &'a $type {
            type Output = Option<&'a $type>;

            fn into_argument(self) -> Option<&'a $type> {
                Some(self)
            }
        }

        impl $crate::ScalarArgument<$type_lock, Option<Vec<$type_lock>>> for Option<Vec<$type>> {
            type Output = Option<Vec<$type>>;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        impl $crate::ScalarArgument<$type_lock, Vec<$type_lock>> for Vec<$type> {
            type Output = Vec<$type>;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        impl $crate::ScalarArgument<$type_lock, Option<Vec<Option<$type_lock>>>>
            for Option<Vec<Option<$type>>>
        {
            type Output = Option<Vec<Option<$type>>>;

            fn into_argument(self) -> Self::Output {
                self
            }
        }

        // TODO: Try and implement list coercion in here...
    };
}

impl_scalar_argument!(i32, i32);
impl_scalar_argument!(f64, f64);
impl_scalar_argument!(String, String);
impl_scalar_argument!(bool, bool);
impl_scalar_argument!(Id, Id);

/*
impl<E, TypeLock> EnumArgument<TypeLock> for E
where
    E: crate::Enum<TypeLock>,
{
    type Output = Self;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<'a, E, TypeLock> EnumArgument<Option<TypeLock>> for Option<&'a E>
where
    E: crate::Enum<TypeLock>,
{
    type Output = Self;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<'a, E, TypeLock> EnumArgument<Option<TypeLock>> for &'a Option<E>
where
    E: crate::Enum<TypeLock>,
{
    type Output = Option<&'a E>;

    fn into_argument(self) -> Self::Output {
        self.as_ref()
    }
}

impl<E, TypeLock> EnumArgument<Option<Vec<TypeLock>>> for Option<Vec<E>>
where
    E: crate::Enum<TypeLock>,
{
    type Output = Option<Vec<E>>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<E, TypeLock> EnumArgument<Option<Vec<Option<TypeLock>>>> for Option<Vec<Option<E>>>
where
    E: crate::Enum<TypeLock>,
{
    type Output = Option<Vec<Option<E>>>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

impl<E, TypeLock> EnumArgument<Vec<TypeLock>> for Vec<E>
where
    E: crate::Enum<TypeLock>,
{
    type Output = Vec<E>;

    fn into_argument(self) -> Self::Output {
        self
    }
}

*/
