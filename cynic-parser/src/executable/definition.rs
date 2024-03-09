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

impl super::Ast {
    pub fn definitions(&self) -> impl Iterator<Item = ExecutableDefinition<'_>> {
        self.definitions.iter().map(|record| match record {
            ExecutableDefinitionRecord::Operation(id) => {
                ExecutableDefinition::Operation(self.read(*id))
            }
            ExecutableDefinitionRecord::Fragment(id) => {
                ExecutableDefinition::Fragment(self.read(*id))
            }
        })
    }
}

impl ExecutableId for ExecutableDefinitionId {
    type Reader<'a> = ExecutableDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ExecutableDefinitionId>> for ExecutableDefinition<'a> {
    fn from(value: ReadContext<'a, ExecutableDefinitionId>) -> Self {
        match value.ast.lookup(value.id) {
            ExecutableDefinitionRecord::Operation(id) => {
                ExecutableDefinition::Operation(value.ast.read(*id))
            }
            ExecutableDefinitionRecord::Fragment(id) => {
                ExecutableDefinition::Fragment(value.ast.read(*id))
            }
        }
    }
}
