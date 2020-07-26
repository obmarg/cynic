// TODO: docs.

use crate::{argument::SerializableArgument, Id};

pub trait IntoArgument<Argument> {
    type Output: SerializableArgument + Send + 'static;

    fn into_argument(self) -> Self::Output;
}

impl<T> IntoArgument<T> for T
where
    T: SerializableArgument + Send + 'static,
{
    type Output = T;

    fn into_argument(self) -> T {
        self
    }
}
/*
impl<T, B> IntoArgument<Option<B>> for T
where
    T: IntoArgument<B>,
{
    type Output = Option<T::Output>;

    fn into_argument(self) -> Option<T::Output> {
        Some(self.into_argument())
    }
}
*/

macro_rules! define_for_owned {
    ($inner:ty) => {
        impl IntoArgument<Option<$inner>> for $inner {
            type Output = Option<$inner>;

            fn into_argument(self) -> Option<$inner> {
                Some(self.clone())
            }
        }
    };
}

macro_rules! define_for_borrow {
    ($inner:ty) => {
        impl IntoArgument<Option<$inner>> for Option<&$inner> {
            type Output = Option<$inner>;

            fn into_argument(self) -> Option<$inner> {
                self.cloned()
            }
        }

        impl IntoArgument<Option<$inner>> for &Option<$inner> {
            type Output = Option<$inner>;

            fn into_argument(self) -> Option<$inner> {
                self.clone()
            }
        }
    };
}

macro_rules! define_for_scalar {
    ($inner:ty) => {
        define_for_owned!($inner);
        define_for_borrow!($inner);
    };
}

define_for_scalar!(i32);
define_for_scalar!(f64);
define_for_scalar!(String);
define_for_scalar!(bool);
define_for_scalar!(Id);

#[cfg(feature = "chrono")]
define_for_scalar!(chrono::FixedOffset);

#[cfg(feature = "chrono")]
define_for_scalar!(chrono::DateTime<chrono::Utc>);

impl IntoArgument<String> for &str {
    type Output = String;

    fn into_argument(self) -> Self::Output {
        self.to_string()
    }
}

impl IntoArgument<Option<String>> for &str {
    type Output = Option<String>;

    fn into_argument(self) -> Self::Output {
        Some(self.to_string())
    }
}

impl IntoArgument<Option<String>> for Option<&str> {
    type Output = Option<String>;

    fn into_argument(self) -> Self::Output {
        self.map(|s| s.to_string())
    }
}
// TODO: Do I also want to define things for Vecs?

// TODO: Define for Enums/InputObjects, though maybe want the derives to take care
//       of that.

// Things I definitely want to accept:
// - T for T.
// - T for Option<T>
// - &T for T
// - &T for Option<T>?
// - DeRefs for T (just manually define these probably)
// Cow etc. ?

// Can't neccesarily use T for some of these but the set of
// T is limited so just implement it manually for those things.
// Essentially scalars, enums, input types
