use indexmap::IndexSet;

pub mod ids;

mod definition;
mod fragment;
mod operation;

pub mod writer;

#[derive(Default)]
pub struct Ast {
    strings: IndexSet<Box<str>>,

    definitions: Vec<definition::ExecutableDefinitionRecord>,
    operations: Vec<operation::OperationDefinitionRecord>,
    fragments: Vec<fragment::FragmentDefinitionRecord>,
}

pub mod storage {
    pub use super::{
        definition::ExecutableDefinitionRecord, fragment::FragmentDefinitionRecord,
        operation::OperationDefinitionRecord,
    };
}
