use super::SelectionSet;

/// A trait for GraphQL enums.
///
/// This trait is generic over some TypeLock which is used to tie an Enum
/// definition back into it's GraphQL enum.  Generally this will be some
/// type generated in the GQL code.
pub trait Enum<TypeLock>: Sized + serde::Serialize {
    /// Returns a `SelectionSet` typed for this enum.
    fn select() -> SelectionSet<'static, Self, TypeLock>;
}
