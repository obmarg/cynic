use super::prelude::*;
use super::{
    directive::Directive,
    ids::{DirectiveId, OperationDefinitionId, SelectionId, VariableDefinitionId},
    selections::Selection,
    variable::VariableDefinition,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct OperationDefinitionRecord {
    pub operation_type: OperationType,
    pub name: Option<StringId>,
    pub variable_definitions: IdRange<VariableDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
}

#[derive(Clone, Copy)]
pub struct OperationDefinition<'a>(pub(in super::super) ReadContext<'a, OperationDefinitionId>);

impl<'a> OperationDefinition<'a> {
    pub fn operation_type(&self) -> OperationType {
        let document = self.0.document;
        document.lookup(self.0.id).operation_type
    }
    pub fn name(&self) -> Option<&'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .name
            .map(|id| document.lookup(id))
    }
    pub fn variable_definitions(&self) -> Iter<'a, VariableDefinition<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).variable_definitions, document)
    }
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn selection_set(&self) -> Iter<'a, Selection<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).selection_set, document)
    }
}

impl OperationDefinition<'_> {
    pub fn id(&self) -> OperationDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for OperationDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OperationDefinition")
            .field("operation_type", &self.operation_type())
            .field("name", &self.name())
            .field("variable_definitions", &self.variable_definitions())
            .field("directives", &self.directives())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl ExecutableId for OperationDefinitionId {
    type Reader<'a> = OperationDefinition<'a>;
    fn read(self, document: &ExecutableDocument) -> Self::Reader<'_> {
        OperationDefinition(ReadContext { id: self, document })
    }
}

impl IdReader for OperationDefinition<'_> {
    type Id = OperationDefinitionId;
}