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

impl<T> IntoInputLiteral<Option<T>> for T
where
    T: IntoInputLiteral<T>,
{
    fn into_literal(self) -> InputLiteral {
        <T as IntoInputLiteral<T>>::into_literal(self)
    }
}

impl<T> IntoInputLiteral<Vec<T>> for T
where
    T: IntoInputLiteral<T>,
{
    fn into_literal(self) -> InputLiteral {
        <T as IntoInputLiteral<T>>::into_literal(self)
    }
}

impl<T> IntoInputLiteral<Option<Vec<T>>> for T
where
    T: IntoInputLiteral<T>,
{
    fn into_literal(self) -> InputLiteral {
        <T as IntoInputLiteral<T>>::into_literal(self)
    }
}

impl<T> IntoInputLiteral<Option<Vec<Option<T>>>> for T
where
    T: IntoInputLiteral<T>,
{
    fn into_literal(self) -> InputLiteral {
        <T as IntoInputLiteral<T>>::into_literal(self)
    }
}

// TODO: Do the other InputLiteral wrappings - Vec<T> into Option<Vec<T>> etc.

// TODO: do the rest of these.
