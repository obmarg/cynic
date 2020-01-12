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


pub trait QueryFragment {
    type SelectionSet: selection_set::Selectable;

    fn selection_set() -> Self::SelectionSet;
}

pub trait QueryRoot {}

// TODO: Think about this API
pub fn to_query<Fragment>() -> String
where
    Fragment: QueryFragment,
    Fragment::SelectionSet: selection_set::Selectable,
    <Fragment::SelectionSet as selection_set::Selectable>::TypeLock: QueryRoot,
{
    use selection_set::Selectable;

    let selection_set = Fragment::selection_set();
    let query = selection_set.query_and_arguments();
    format!("{:?}", query)
}

pub use cynic_codegen::{query_dsl, QueryFragment};
