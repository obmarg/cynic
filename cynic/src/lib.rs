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

    fn selection_set() -> Self::SelectionSet;
}

pub trait QueryRoot {}

// TODO: THink about this API
pub fn to_query<'a, Fragment>() -> String
where
    Fragment: QueryFragment<'a>,
    Fragment::SelectionSet: selection_set::Selectable,
    <Fragment::SelectionSet as selection_set::Selectable>::TypeLock: QueryRoot,
{
    use selection_set::Selectable;

    let selection_set = Fragment::selection_set();
    let query = selection_set.query_and_arguments();
    format!("{:?}", query)
}

pub use cynic_codegen::{query_dsl, QueryFragment};
