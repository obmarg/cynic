use super::SelectionSet;

pub trait QueryFragment {
    type SelectionSet;
    type Arguments: FragmentArguments;

    fn fragment(arguments: FragmentContext<Self::Arguments>) -> Self::SelectionSet;
    fn graphql_type() -> String;
}

pub trait InlineFragments: Sized {
    type TypeLock;
    type Arguments: FragmentArguments;

    fn graphql_type() -> String;
    fn fragments(
        arguments: FragmentContext<Self::Arguments>,
    ) -> Vec<(String, SelectionSet<'static, Self, Self::TypeLock>)>;
}

impl<T> QueryFragment for T
where
    T: InlineFragments + Send + Sync + 'static,
{
    type SelectionSet = SelectionSet<'static, T, T::TypeLock>;
    type Arguments = <T as InlineFragments>::Arguments;

    fn fragment(arguments: FragmentContext<Self::Arguments>) -> Self::SelectionSet {
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

// TODO: Docs, and think about the name?  FragmentContext/QueryContext/Context/something else?
pub struct FragmentContext<'a, Args> {
    pub args: &'a Args,
}

impl<'a, Args> FragmentContext<'a, Args> {
    pub fn with_args(args: &'a Args) -> FragmentContext<'a, Args> {
        FragmentContext { args }
    }
}

impl FragmentContext<'static, ()> {
    // TODO: Think about this name: empty? without_args? no_args?
    pub fn empty() -> FragmentContext<'static, ()> {
        FragmentContext { args: &() }
    }
}
