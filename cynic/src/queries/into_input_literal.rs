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

#[macro_export]
macro_rules! impl_into_input_literal_for_wrappers {
    ($target:ty) => {
        impl_into_input_literal_for_wrappers!($target, $target);
    };
    ($target:ty, $typelock:ty) => {
        impl IntoInputLiteral<Option<$typelock>> for $target {
            fn into_literal(self) -> InputLiteral {
                <$target as IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        impl IntoInputLiteral<Vec<$typelock>> for $target {
            fn into_literal(self) -> InputLiteral {
                <$target as IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        impl IntoInputLiteral<Option<Vec<$typelock>>> for $target {
            fn into_literal(self) -> InputLiteral {
                <$target as IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        impl IntoInputLiteral<Option<Vec<Option<$typelock>>>> for $target {
            fn into_literal(self) -> InputLiteral {
                <$target as IntoInputLiteral<$typelock>>::into_literal(self)
            }
        }

        // TODO: impl all the other variants...

        impl IntoInputLiteral<Option<$typelock>> for Option<$target> {
            fn into_literal(self) -> InputLiteral {
                match self {
                    None => InputLiteral::Null,
                    Some(inner) => <$target as IntoInputLiteral<$typelock>>::into_literal(inner),
                }
            }
        }
    };
}

mod scalars {
    use super::*;

    impl IntoInputLiteral<i32> for i32 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Int(self)
        }
    }
    impl_into_input_literal_for_wrappers!(i32);

    impl IntoInputLiteral<f64> for f64 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Float(self)
        }
    }
    impl_into_input_literal_for_wrappers!(f64);

    impl IntoInputLiteral<bool> for bool {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Bool(self)
        }
    }
    impl_into_input_literal_for_wrappers!(bool);

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
    impl_into_input_literal_for_wrappers!(String);

    impl IntoInputLiteral<Id> for Id {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Id(self.into_inner())
        }
    }
    impl_into_input_literal_for_wrappers!(Id);
}

// TODO: Do the other InputLiteral wrappings - Vec<T> into Option<Vec<T>> etc.

// TODO: do the rest of these.
