mod argument;
mod field;
mod scalar;
pub mod selection_set;

pub use argument::Argument;
pub use scalar::Scalar;
pub use selection_set::SelectionSet;

fn main() {
    println!("Hello, world!");
}

// Ok, so I need a way of figuring out TypeLocks.
// I _could_ do QueryFragment<T>
// and then do impl<T> QueryFragment<T> for X
// Though I'd need a way to constrain T.
//
// Perhaps with some kind of marker trait?
// Contains<T>?

pub trait QueryFragment<'a> {
    type SelectionSet: selection_set::Selectable;
    type Arguments: FragmentArguments;

    // TODO: Need an argument type here...

    // Selection set needs to implement the argument type.

    // And we provide some sort of conversion between argument
    // types.  Maybe From?  Maybe something else...
    fn selection_set(arguments: Self::Arguments) -> Self::SelectionSet;
}

/// A marker trait for the arguments types on QueryFragments.
///
/// We use this in combination with the IntoArguments trait below
/// to convert between different argument types in a query heirarchy.
pub trait FragmentArguments {}

impl FragmentArguments for () {}

/// Used for converting between different argument types in a QueryFragment
/// heirarchy.
///
/// For example if an outer QueryFragment has a struct with several parameters
/// but an inner QueryFragment needs none then we can use () as the arguments
/// type on the inner fragments and use the blanket implementation of IntoArguments
/// to convert to ().
///
/// Similarly, the
pub trait IntoArguments<T> {
    fn into_args(&self) -> T;
}

impl IntoArguments<()> for dyn FragmentArguments {
    fn into_args(&self) -> () {
        ()
    }
}

impl<T> IntoArguments<T> for T
where
    T: Clone,
{
    fn into_args(&self) -> T {
        // TODO: Figure out if there's a way to avoid this clone...
        self.clone()
    }
}

pub trait QueryRoot {}

// TODO: THink about this API
pub fn to_query<'a, Fragment>(args: Fragment::Arguments) -> String
where
    Fragment: QueryFragment<'a>,
    Fragment::SelectionSet: selection_set::Selectable,
    <Fragment::SelectionSet as selection_set::Selectable>::TypeLock: QueryRoot,
{
    use selection_set::Selectable;

    let selection_set = Fragment::selection_set(args);
    let query = selection_set.query_and_arguments();
    format!("{:?}", query)
}

pub use cynic_codegen::{query_dsl, QueryFragment};
