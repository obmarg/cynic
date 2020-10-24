use super::SelectionSet;

pub trait QueryFragment {
    type SelectionSet;
    type Arguments: FragmentArguments;

    // Ok, so what if this takes an `Into<FragmentContext>` instead.
    // We introduce a struct FragmentContext<Arguments> type.
    // impl Into<FragmentContext> for all args (or maybe just get users to convert manually?)
    // Then use that to propagate any metadata around recursion.
    fn fragment(arguments: &Self::Arguments) -> Self::SelectionSet;
    fn graphql_type() -> String;
}

pub trait InlineFragments: Sized {
    type TypeLock;
    type Arguments: FragmentArguments;

    fn graphql_type() -> String;
    fn fragments(
        arguments: &Self::Arguments,
    ) -> Vec<(String, SelectionSet<'static, Self, Self::TypeLock>)>;
}

impl<T> QueryFragment for T
where
    T: InlineFragments + Send + Sync + 'static,
{
    type SelectionSet = SelectionSet<'static, T, T::TypeLock>;
    type Arguments = <T as InlineFragments>::Arguments;

    fn fragment(arguments: &Self::Arguments) -> Self::SelectionSet {
        crate::selection_set::inline_fragments(Self::fragments(arguments))
    }

    fn graphql_type() -> String {
        Self::graphql_type()
    }
}

/// A marker trait for the arguments types on QueryFragments.
///
/// We use this in combination with the IntoArguments trait below
/// to convert between different argument types in a query hierarchy.
pub trait FragmentArguments {}

impl FragmentArguments for () {}
