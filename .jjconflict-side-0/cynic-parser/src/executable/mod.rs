use indexmap::IndexSet;

pub mod ids;
pub mod iter;
pub mod writer;

mod extensions;
mod generated;
mod types;
mod value;

use crate::common::IdRange;

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

use self::{ids::ExecutableDefinitionId, iter::Iter};

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

// TODO: Make this sealed maybe?  Could also move into id module...
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
    pub fn definitions(&self) -> Iter<'_, ExecutableDefinition<'_>> {
        Iter::new(
            IdRange::new(
                ExecutableDefinitionId::new(0),
                ExecutableDefinitionId::new(self.definitions.len()),
            ),
            self,
        )
    }

    pub fn operations(&self) -> impl DoubleEndedIterator<Item = OperationDefinition<'_>> + '_ {
        self.definitions().filter_map(|op| match op {
            ExecutableDefinition::Operation(reader) => Some(reader),
            ExecutableDefinition::Fragment(_) => None,
        })
    }

    pub fn fragments(&self) -> impl DoubleEndedIterator<Item = FragmentDefinition<'_>> + '_ {
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
