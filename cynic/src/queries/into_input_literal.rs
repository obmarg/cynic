use super::InputLiteral;
use crate::Id;

pub trait IntoInputLiteral {
    fn into_literal(self) -> InputLiteral;
}

impl<T> IntoInputLiteral for Option<T>
where
    T: IntoInputLiteral,
{
    fn into_literal(self) -> InputLiteral {
        match self {
            None => InputLiteral::Null,
            Some(inner) => <T as IntoInputLiteral>::into_literal(inner),
        }
    }
}

impl<T> IntoInputLiteral for Vec<T>
where
    T: IntoInputLiteral,
{
    fn into_literal(self) -> InputLiteral {
        InputLiteral::List(self.into_iter().map(T::into_literal).collect())
    }
}

mod scalars {
    use super::*;

    impl IntoInputLiteral for i32 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Int(self)
        }
    }

    impl IntoInputLiteral for f64 {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Float(self)
        }
    }

    impl IntoInputLiteral for bool {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Bool(self)
        }
    }

    impl IntoInputLiteral for String {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::String(self.into())
        }
    }

    impl IntoInputLiteral for &str {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::String(self.to_string().into())
        }
    }

    impl IntoInputLiteral for Id {
        fn into_literal(self) -> InputLiteral {
            InputLiteral::Id(self.into_inner())
        }
    }
}

// TODO: Do the other InputLiteral wrappings - Vec<T> into Option<Vec<T>> etc.

// TODO: do the rest of these.
