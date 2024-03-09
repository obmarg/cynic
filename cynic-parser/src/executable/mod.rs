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

#[cfg(feature = "print")]
mod printer;

pub use self::{
    argument::Argument,
    definition::ExecutableDefinition,
    directive::Directive,
    fragment::FragmentDefinition,
    operation::OperationDefinition,
    selections::{FieldSelection, FragmentSpread, InlineFragment, Selection},
    types::Type,
    value::Value,
    variable::VariableDefinition,
};

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

// TODO: Make this sealed maybe?
pub trait ExecutableId: Copy {
    type Reader<'a>: From<ReadContext<'a, Self>>;

    fn read(self, ast: &Ast) -> Self::Reader<'_> {
        ReadContext { id: self, ast }.into()
    }
}

#[derive(Clone, Copy)]
pub struct ReadContext<'a, I> {
    id: I,
    ast: &'a Ast,
}

impl Ast {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: ExecutableId,
    {
        ReadContext { id, ast: self }.into()
    }
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
