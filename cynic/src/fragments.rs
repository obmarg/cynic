use super::SelectionSet;

/// Indicates that a type may be used as part of a graphql query.
///
/// This will usually be derived, but can be manually implemented if required.
pub trait QueryFragment {
    /// The type of `SelectionSet` that is output by this `QueryFragment`
    type SelectionSet;

    /// The arguments that are required to select this fragment.
    type Arguments: FragmentArguments;

    /// Returns a `SelectionSet` that selects this `QueryFragment` with the arguments provided
    /// in the `context`
    fn fragment(context: FragmentContext<Self::Arguments>) -> Self::SelectionSet;

    /// The GraphQL type name that this `QueryFragment` selects
    fn graphql_type() -> String;
}

/// Indicates that a type may be used to select one or more members of an interface/union type.
///
/// Similar to a collection of inline fragments in a GraphQL query.
pub trait InlineFragments: Sized {
    /// The type lock for the GraphQL interface/union type these fragments are selected on.
    type TypeLock;

    /// The arguments that are required to select these fragments.
    type Arguments: FragmentArguments;

    /// The GraphQL type name that this `InlineFragments` selects
    fn graphql_type() -> String;

    /// Returns a `SelectionSet` that selects the fragments for this type with the arguments provided
    /// in the `context`
    fn fragments(
        context: FragmentContext<Self::Arguments>,
    ) -> Vec<(String, SelectionSet<'static, Self, Self::TypeLock>)>;

    /// Returns an optional fallback `SelectionSet` for when the returned type does not match
    /// any of the types supported by this `InlineFragments`
    fn fallback(
        context: FragmentContext<Self::Arguments>,
    ) -> Option<SelectionSet<'static, Self, Self::TypeLock>>;
}

impl<T> QueryFragment for T
where
    T: InlineFragments + Send + Sync + 'static,
{
    type SelectionSet = SelectionSet<'static, T, T::TypeLock>;
    type Arguments = <T as InlineFragments>::Arguments;

    fn fragment(context: FragmentContext<Self::Arguments>) -> Self::SelectionSet {
        crate::selection_set::inline_fragments(
            Self::fragments(context.clone()),
            Self::fallback(context),
        )
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
    /// The `FragmentArguments` for this part of the query.
    pub args: &'a Args,

    /// The current recurse depth (if any) for building a query
    ///
    /// This is used when building a recursive query to make sure we know
    /// when to stop recursing.
    pub recurse_depth: Option<u8>,
}

impl<'a, Args> Clone for FragmentContext<'a, Args> {
    fn clone(&self) -> Self {
        FragmentContext {
            args: self.args,
            recurse_depth: self.recurse_depth,
        }
    }
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
            recurse_depth: self.recurse_depth,
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
