use std::borrow::Cow;

use super::InputLiteral;

pub trait IntoInputLiteral<SchemaType> {
    fn into_literal(self) -> InputLiteral;
}

impl IntoInputLiteral<crate::Id> for crate::Id {
    fn into_literal(self) -> InputLiteral {
        InputLiteral::Id(self.into_inner())
    }
}

// TODO: Mostly putting this particular impl here to shut up the compiler temporarily.
// Should probably remove it once i've updated the argument literal code to
// construct IDs.
impl IntoInputLiteral<crate::Id> for &str {
    fn into_literal(self) -> InputLiteral {
        InputLiteral::String(Cow::Owned(self.to_string()))
    }
}

macro_rules! def_coercions {
    ($target:ty) => {
        def_coercions!($target, $target);
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
    };
}

mod scalars {
    use super::*;

    impl IntoInputLiteral<i32> for i32 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Int(self)
        }
    }
    def_coercions!(i32);

    impl IntoInputLiteral<f64> for f64 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Float(self)
        }
    }
    def_coercions!(f64);

    impl IntoInputLiteral<bool> for bool {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Bool(self)
        }
    }
    def_coercions!(bool);

    impl IntoInputLiteral<String> for &str {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::String(self.to_string().into())
        }
    }
    def_coercions!(&str, String);

    impl IntoInputLiteral<String> for String {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::String(self.into())
        }
    }
    def_coercions!(String);
}

// TODO: Do the other InputLiteral wrappings - Vec<T> into Option<Vec<T>> etc.

// TODO: do the rest of these.
