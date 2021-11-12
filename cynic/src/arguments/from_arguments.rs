use crate::FragmentArguments;

/// Used for converting between different argument types in a QueryFragment
/// hierarchy.
///
/// For example if an outer QueryFragment has a struct with several parameters
/// but an inner QueryFragment needs none then we can use () as the arguments
/// type on the inner fragments and use the blanket implementation of FromArguments
/// to convert to ().
pub trait FromArguments<T> {
    /// Converts a `T` into `Self`
    fn from_arguments(args: T) -> Self;
}

impl<'a, T> FromArguments<&'a T> for &'a T
where
    T: FragmentArguments,
{
    fn from_arguments(args: &'a T) -> Self {
        args
    }
}
