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
}

impl IdReader for ExecutableDefinition<'_> {
    type Id = ExecutableDefinitionId;
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
