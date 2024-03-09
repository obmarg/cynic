use crate::AstLookup;

use super::{
    fragment::FragmentDefinition,
    ids::{ExecutableDefinitionId, FragmentDefinitionId, OperationDefinitionId},
    operation::OperationDefinition,
    ExecutableId, ReadContext,
};

pub enum ExecutableDefinitionRecord {
    Operation(OperationDefinitionId),
    Fragment(FragmentDefinitionId),
}

#[derive(Clone, Copy)]
pub enum ExecutableDefinition<'a> {
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
}

impl super::ExecutableDocument {
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = ExecutableDefinition<'_>> {
        self.definitions.iter().map(|record| match record {
            ExecutableDefinitionRecord::Operation(id) => {
                ExecutableDefinition::Operation(self.read(*id))
            }
            ExecutableDefinitionRecord::Fragment(id) => {
                ExecutableDefinition::Fragment(self.read(*id))
            }
        })
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

impl ExecutableId for ExecutableDefinitionId {
    type Reader<'a> = ExecutableDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ExecutableDefinitionId>> for ExecutableDefinition<'a> {
    fn from(value: ReadContext<'a, ExecutableDefinitionId>) -> Self {
        match value.document.lookup(value.id) {
            ExecutableDefinitionRecord::Operation(id) => {
                ExecutableDefinition::Operation(value.document.read(*id))
            }
            ExecutableDefinitionRecord::Fragment(id) => {
                ExecutableDefinition::Fragment(value.document.read(*id))
            }
        }
    }
}
