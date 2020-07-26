// TODO: docs.
use std::borrow::Cow;

use crate::{argument::SerializableArgument, Id};

pub trait IntoArgument<'a, Argument> {
    type Output: SerializableArgument + Send + 'a;

    fn into_argument(self) -> Self::Output;
}

/*
pub trait IntoArgument2<'a, Argument> {
    fn into_argument(&'a self) -> &'a Argument;
    // Think this is problematic because lifetimes become required _even if_ we're
    // taking ownership
}
*/

impl<'a, T, B> IntoArgument<'a, Option<B>> for T
where
    T: IntoArgument<'a, B>,
{
    type Output = Option<T::Output>;

    fn into_argument(self) -> Option<T::Output> {
        Some(self.into_argument())
    }
}

macro_rules! define_for_owned {
    ($inner:ty) => {
        impl IntoArgument<'static, $inner> for $inner {
            type Output = $inner;

            fn into_argument(self) -> $inner {
                self
            }
        }

        impl IntoArgument<'static, Option<$inner>> for Option<$inner> {
            type Output = Option<$inner>;

            fn into_argument(self) -> Option<$inner> {
                self
            }
        }
    };
}

macro_rules! define_for_borrow {
    ($inner:ty) => {
        impl<'a> IntoArgument<'a, $inner> for &'a $inner {
            type Output = &'a $inner;

            fn into_argument(self) -> &'a $inner {
                self
            }
        }

        impl<'a> IntoArgument<'a, Option<$inner>> for Option<&'a $inner> {
            type Output = Option<&'a $inner>;

            fn into_argument(self) -> Option<&'a $inner> {
                self
            }
        }

        // TODO: probably also want &'a Option<inner>?
    };
}

macro_rules! define_for_scalar {
    ($inner:ty) => {
        define_for_owned!($inner);
        define_for_borrow!($inner);
    };
}

define_for_scalar!(i32);
define_for_scalar!(String);
define_for_scalar!(Id);

// TODO: Can I take advantage of the fact that there's a limited
// subset of things that can be arguments here, and actually enumerate
// every possibility rather than adding this generic impl.
// This would give me a lot more leeway to do stuff with AsRef etc.

// Things that can be arguments: scalars, input types, vecs<other_args>, options<other_args>
// Actually very simple.
// Also worth noting that these are the only types that need to be serialized, and
// _also_ currently the only types SerializableArgument are implemented for...
/*
impl<T> IntoArgument<T> for T {
    fn into_argument(self) -> T {
        self
    }
}*/

// TODO: Ok, so ideas:
// Maybe do an IntoArgument<T> for T
// Then just be specific about all the conversions we want to support,
// using macros to cut down on the pain of defining all of them...

impl<'a> IntoArgument<'a, String> for &'a str {
    type Output = &'a str;

    fn into_argument(self) -> &'a str {
        self
    }
}

impl<'a> IntoArgument<'a, Option<String>> for Option<&'a str> {
    type Output = Option<&'a str>;

    fn into_argument(self) -> Option<&'a str> {
        self
    }
}

// Things I want to accept:
// - T for T.
// - T for Option<T>
// - &T for T
// - &T for Option<T>?
// - DeRefs for T (just manually define these probably)
// Cow etc. ?

// Can't neccesarily use T for some of these but the set of
// T is limited so just implement it manually for those things.
// Essentially scalars, enums, input types
