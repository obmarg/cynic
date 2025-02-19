/// Encodes the rules for what types can be flattened into other types
/// via the `#[cynic(flatten)]` attribute.
///
/// This trait is sealed and can't be implemented by users of cynic.
#[diagnostic::on_unimplemented(
    message = "{Self} cannot be flattened into a {T}",
    label = "In the GraphQL schema this field has type {Self}.  We can't flatten that type into {T}",
    note = "Change this fields type to be closer to {Self}"
)]
pub trait FlattensInto<T>: private::Sealed<T> {}

impl<T> FlattensInto<Vec<T>> for Vec<Option<T>> {}
impl<T> FlattensInto<Vec<T>> for Option<Vec<T>> {}
impl<T> FlattensInto<Option<Vec<T>>> for Option<Vec<Option<T>>> {}
impl<T> FlattensInto<Vec<T>> for Option<Vec<Option<T>>> {}

mod private {
    pub trait Sealed<T> {}

    impl<T> Sealed<Vec<T>> for Vec<Option<T>> {}
    impl<T> Sealed<Vec<T>> for Option<Vec<T>> {}
    impl<T> Sealed<Option<Vec<T>>> for Option<Vec<Option<T>>> {}
    impl<T> Sealed<Vec<T>> for Option<Vec<Option<T>>> {}
}
