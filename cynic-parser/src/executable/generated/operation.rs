#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directive::Directive,
    ids::{DirectiveId, OperationDefinitionId, SelectionId, VariableDefinitionId},
    selections::Selection,
    variable::VariableDefinition,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
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
pub struct OperationDefinition<'a>(ReadContext<'a, OperationDefinitionId>);

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
    pub fn variable_definitions(
        &self,
    ) -> impl ExactSizeIterator<Item = VariableDefinition<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .variable_definitions
            .iter()
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
    }
    pub fn selection_set(&self) -> impl ExactSizeIterator<Item = Selection<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .selection_set
            .iter()
            .map(|id| document.read(id))
    }
}

impl fmt::Debug for OperationDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OperationDefinition")
            .field("operation_type", &self.operation_type())
            .field("name", &self.name())
            .field(
                "variable_definitions",
                &self.variable_definitions().collect::<Vec<_>>(),
            )
            .field("directives", &self.directives().collect::<Vec<_>>())
            .field("selection_set", &self.selection_set().collect::<Vec<_>>())
            .finish()
    }
}

impl ExecutableId for OperationDefinitionId {
    type Reader<'a> = OperationDefinition<'a>;
}

impl<'a> From<ReadContext<'a, OperationDefinitionId>> for OperationDefinition<'a> {
    fn from(value: ReadContext<'a, OperationDefinitionId>) -> Self {
        Self(value)
    }
}
