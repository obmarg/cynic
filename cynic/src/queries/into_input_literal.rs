use std::borrow::Cow;

use super::InputLiteral;
use crate::Id;

pub trait IntoInputLiteral<SchemaType> {
    fn into_literal(self) -> InputLiteral;
}

// TODO: Mostly putting this particular impl here to shut up the compiler temporarily.
// Should probably remove it once i've updated the argument literal code to
// construct IDs.
impl IntoInputLiteral<Id> for &str {
    fn into_literal(self) -> InputLiteral {
        InputLiteral::String(Cow::Owned(self.to_string()))
    }
}

impl<T, TypeLock> IntoInputLiteral<Option<TypeLock>> for Option<T>
where
    T: IntoInputLiteral<TypeLock>,
{
    fn into_literal(self) -> InputLiteral {
        match self {
            None => InputLiteral::Null,
            Some(inner) => <T as IntoInputLiteral<TypeLock>>::into_literal(inner),
        }
    }
}

#[macro_export]
macro_rules! impl_into_input_literal_for_wrappers {
    ($target:ty, $typelock:ty) => {
        impl $crate::queries::IntoInputLiteral<Option<$typelock>> for $target {
            fn into_literal(self) -> $crate::queries::InputLiteral {
                <$target as $crate::queries::IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        impl $crate::queries::IntoInputLiteral<Vec<$typelock>> for $target {
            fn into_literal(self) -> $crate::queries::InputLiteral {
                <$target as $crate::queries::IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        impl $crate::queries::IntoInputLiteral<Option<Vec<$typelock>>> for $target {
            fn into_literal(self) -> $crate::queries::InputLiteral {
                <$target as $crate::queries::IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        impl $crate::queries::IntoInputLiteral<Option<Vec<Option<$typelock>>>> for $target {
            fn into_literal(self) -> $crate::queries::InputLiteral {
                <$target as $crate::queries::IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        // TODO: impl all the other variants...

        // impl $crate::queries::IntoInputLiteral<Option<$typelock>> for Option<$target> {
        //     fn into_literal(self) -> $crate::queries::InputLiteral {
        //         match self {
        //             None => $crate::queries::InputLiteral::Null,
        //             Some(inner) => {
        //                 <$target as $crate::queries::IntoInputLiteral<$typelock>>::into_literal(
        //                     inner,
        //                 )
        //             }
        //         }
        //     }
        // }
    };
}

macro_rules! impl_into_input_literal_for_scalar_wrappers {
    ($target:ty) => {
        crate::impl_into_input_literal_for_wrappers!($target, $target);
    };
}

mod scalars {
    use super::*;

    impl IntoInputLiteral<i32> for i32 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Int(self)
        }
    }
    impl_into_input_literal_for_scalar_wrappers!(i32);

    impl IntoInputLiteral<f64> for f64 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Float(self)
        }
    }
    impl_into_input_literal_for_scalar_wrappers!(f64);

    impl IntoInputLiteral<bool> for bool {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Bool(self)
        }
    }
    impl_into_input_literal_for_scalar_wrappers!(bool);

    impl IntoInputLiteral<String> for &str {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::String(self.to_string().into())
        }
    }
    impl_into_input_literal_for_wrappers!(&str, String);

    impl IntoInputLiteral<String> for String {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::String(self.into())
        }
    }
    impl_into_input_literal_for_scalar_wrappers!(String);

    impl IntoInputLiteral<Id> for Id {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Id(self.into_inner())
        }
    }
    impl_into_input_literal_for_scalar_wrappers!(Id);
}

// TODO: Do the other InputLiteral wrappings - Vec<T> into Option<Vec<T>> etc.

// TODO: do the rest of these.
