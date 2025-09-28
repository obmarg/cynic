use super::prelude::*;
use super::{
    ExecutableId,
    directive::Directive,
    ids::{DirectiveId, OperationDefinitionId, SelectionId, VariableDefinitionId},
    selections::Selection,
    variable::VariableDefinition,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct OperationDefinitionRecord {
    pub operation_type: OperationType,
    pub operation_type_span: Option<Span>,
    pub name: Option<StringId>,
    pub name_span: Option<Span>,
    pub variable_definitions: IdRange<VariableDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub selection_set: IdRange<SelectionId>,
    pub selection_set_span: Span,
}

#[derive(Clone, Copy)]
pub struct OperationDefinition<'a>(pub(in super::super) ReadContext<'a, OperationDefinitionId>);

impl<'a> OperationDefinition<'a> {
    pub fn operation_type(&self) -> OperationType {
        let document = self.0.document;
        document.lookup(self.0.id).operation_type
    }
    pub fn operation_type_span(&self) -> Option<Span> {
        let document = self.0.document;
        document.lookup(self.0.id).operation_type_span
    }
    pub fn name(&self) -> Option<&'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .name
            .map(|id| document.lookup(id))
    }
    pub fn name_span(&self) -> Option<Span> {
        let document = self.0.document;
        document.lookup(self.0.id).name_span
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
    pub fn selection_set_span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).selection_set_span
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
    type Reader<'a> = OperationDefinition<'a>;
    fn new(id: Self::Id, document: &'_ ExecutableDocument) -> Self::Reader<'_> {
        document.read(id)
    }
}
