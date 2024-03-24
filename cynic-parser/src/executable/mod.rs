use indexmap::IndexSet;

pub mod ids;

mod generated;
mod value;

mod types;
pub mod writer;

use self::ids::ExecutableDefinitionId;
pub use self::{
    generated::{
        argument::Argument,
        definition::ExecutableDefinition,
        directive::Directive,
        fragment::FragmentDefinition,
        operation::OperationDefinition,
        selections::{FieldSelection, FragmentSpread, InlineFragment, Selection},
        variable::VariableDefinition,
    },
    types::Type,
    value::Value,
};

#[derive(Default)]
pub struct ExecutableDocument {
    strings: IndexSet<Box<str>>,
    block_strings: Vec<Box<str>>,

    definitions: Vec<storage::ExecutableDefinitionRecord>,
    operations: Vec<storage::OperationDefinitionRecord>,
    fragments: Vec<storage::FragmentDefinitionRecord>,

    selections: Vec<storage::SelectionRecord>,
    field_selections: Vec<storage::FieldSelectionRecord>,
    inline_fragments: Vec<storage::InlineFragmentRecord>,
    fragment_spreads: Vec<storage::FragmentSpreadRecord>,

    directives: Vec<storage::DirectiveRecord>,
    arguments: Vec<storage::ArgumentRecord>,
    variables: Vec<storage::VariableDefinitionRecord>,

    types: Vec<types::TypeRecord>,

    values: Vec<value::ValueRecord>,
}

// TODO: Make this sealed maybe?
pub trait ExecutableId: Copy {
    type Reader<'a>: From<ReadContext<'a, Self>>;

    fn read(self, ast: &ExecutableDocument) -> Self::Reader<'_> {
        ReadContext {
            id: self,
            document: ast,
        }
        .into()
    }
}

#[derive(Clone, Copy)]
pub struct ReadContext<'a, I> {
    id: I,
    document: &'a ExecutableDocument,
}

impl ExecutableDocument {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: ExecutableId,
    {
        ReadContext { id, document: self }.into()
    }
}

impl ExecutableDocument {
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = ExecutableDefinition<'_>> {
        self.definitions
            .iter()
            .enumerate()
            .map(|(i, _)| self.read(ExecutableDefinitionId::new(i)))
    }

    pub fn operations(&self) -> impl Iterator<Item = OperationDefinition<'_>> {
        self.definitions().filter_map(|op| match op {
            ExecutableDefinition::Operation(reader) => Some(reader),
            ExecutableDefinition::Fragment(_) => None,
        })
    }

    pub fn fragments(&self) -> impl Iterator<Item = FragmentDefinition<'_>> {
        self.definitions().filter_map(|op| match op {
            ExecutableDefinition::Operation(_) => None,
            ExecutableDefinition::Fragment(reader) => Some(reader),
        })
    }
}

pub mod storage {
    pub use super::{
        generated::{
            argument::ArgumentRecord,
            definition::ExecutableDefinitionRecord,
            directive::DirectiveRecord,
            fragment::FragmentDefinitionRecord,
            operation::OperationDefinitionRecord,
            selections::{
                FieldSelectionRecord, FragmentSpreadRecord, InlineFragmentRecord, SelectionRecord,
            },
            variable::VariableDefinitionRecord,
        },
        types::TypeRecord,
        value::ValueRecord,
    };
}
