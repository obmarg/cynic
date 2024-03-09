use indexmap::IndexSet;

pub mod ids;

mod definition;
mod fragment;
mod operation;
mod selections;
mod value;

mod argument;
mod directive;
pub mod writer;

#[derive(Default)]
pub struct Ast {
    strings: IndexSet<Box<str>>,

    definitions: Vec<definition::ExecutableDefinitionRecord>,
    operations: Vec<operation::OperationDefinitionRecord>,
    fragments: Vec<fragment::FragmentDefinitionRecord>,

    selections: Vec<selections::SelectionRecord>,
    field_selections: Vec<selections::FieldSelectionRecord>,
    inline_fragments: Vec<selections::InlineFragmentRecord>,
    fragment_spreads: Vec<selections::FragmentSpreadRecord>,

    directives: Vec<directive::DirectiveRecord>,
    arguments: Vec<argument::ArgumentRecord>,

    values: Vec<value::ValueRecord>,
}

pub mod storage {
    pub use super::{
        argument::ArgumentRecord,
        definition::ExecutableDefinitionRecord,
        directive::DirectiveRecord,
        fragment::FragmentDefinitionRecord,
        operation::OperationDefinitionRecord,
        selections::{
            FieldSelectionRecord, FragmentSpreadRecord, InlineFragmentRecord, SelectionRecord,
        },
        value::ValueRecord,
    };
}
