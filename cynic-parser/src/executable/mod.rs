use indexmap::IndexSet;

pub mod ids;

mod argument;
mod definition;
mod directive;
mod fragment;
mod operation;
mod selections;
mod value;
mod variable;

mod types;
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
    variables: Vec<variable::VariableDefinitionRecord>,

    types: Vec<types::TypeRecord>,

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
        types::TypeRecord,
        value::ValueRecord,
        variable::VariableDefinitionRecord,
    };
}
