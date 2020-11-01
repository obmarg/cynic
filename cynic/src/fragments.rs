use super::SelectionSet;

pub trait QueryFragment {
    type SelectionSet;
    type Arguments: FragmentArguments;

    fn fragment(context: FragmentContext<Self::Arguments>) -> Self::SelectionSet;
    fn graphql_type() -> String;
}

pub trait InlineFragments: Sized {
    type TypeLock;
    type Arguments: FragmentArguments;

    fn graphql_type() -> String;
    fn fragments(
        context: FragmentContext<Self::Arguments>,
    ) -> Vec<(String, SelectionSet<'static, Self, Self::TypeLock>)>;
}

impl<T> QueryFragment for T
where
    T: InlineFragments + Send + Sync + 'static,
{
    type SelectionSet = SelectionSet<'static, T, T::TypeLock>;
    type Arguments = <T as InlineFragments>::Arguments;

    fn fragment(context: FragmentContext<Self::Arguments>) -> Self::SelectionSet {
        crate::selection_set::inline_fragments(Self::fragments(context))
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

/// Context passed into a QueryFragment/InlineFragments
///
/// This contains the arguments to be used by the fragment and other
/// metadata neccesary for building the fragment.
///
/// Should be built with the `new` function to pass in arguments or the
/// `empty` function if there are no arguments.
pub struct FragmentContext<'a, Args> {
    pub args: &'a Args,
    pub recurse_depth: Option<u8>,
}

impl<'a, Args> FragmentContext<'a, Args> {
    /// Constructs a new FragmentContext with some arguments.
    ///
    /// The `empty` function can be used instead if there are no arguments.
    pub fn new(args: &'a Args) -> FragmentContext<'a, Args> {
        FragmentContext {
            args,
            recurse_depth: None,
        }
    }

    #[doc(hidden)]
    pub fn with_args<'b, NewArgs>(&self, args: &'b NewArgs) -> FragmentContext<'b, NewArgs> {
        FragmentContext {
            args,
            recurse_depth: self.recurse_depth.clone(),
        }
    }

    #[doc(hidden)]
    pub fn recurse(&self) -> Self {
        FragmentContext {
            recurse_depth: self.recurse_depth.or(Some(0)).map(|d| d + 1),
            args: self.args,
        }
    }
}

impl FragmentContext<'static, ()> {
    /// Constructs a new FragmentContext with no arguments
    pub fn empty() -> FragmentContext<'static, ()> {
        FragmentContext {
            args: &(),
            recurse_depth: None,
        }
    }
}
