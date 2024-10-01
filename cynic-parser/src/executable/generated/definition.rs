use super::prelude::*;
use super::{
    fragment::FragmentDefinition,
    ids::{ExecutableDefinitionId, FragmentDefinitionId, OperationDefinitionId},
    operation::OperationDefinition,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub enum ExecutableDefinitionRecord {
    Operation(OperationDefinitionId),
    Fragment(FragmentDefinitionId),
}

#[derive(Clone, Copy, Debug)]
pub enum ExecutableDefinition<'a> {
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
}

impl ExecutableId for ExecutableDefinitionId {
    type Reader<'a> = ExecutableDefinition<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        match document.lookup(self) {
            ExecutableDefinitionRecord::Operation(id) => {
                ExecutableDefinition::Operation(document.read(*id))
            }
            ExecutableDefinitionRecord::Fragment(id) => {
                ExecutableDefinition::Fragment(document.read(*id))
            }
        }
    }
}

impl IdReader for ExecutableDefinition<'_> {
    type Id = ExecutableDefinitionId;
}