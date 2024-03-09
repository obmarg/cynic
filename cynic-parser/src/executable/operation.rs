use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

use super::{
    directive::Directive,
    ids::{DirectiveId, OperationDefinitionId, SelectionId, StringId, VariableDefinitionId},
    selections::Selection,
    variable::VariableDefinition,
    ExecutableId, ReadContext,
};

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
        let ast = &self.0.ast;
        ast.lookup(self.0.id).operation_type
    }

    pub fn name(&self) -> Option<&'a str> {
        let ast = &self.0.ast;
        ast.lookup(self.0.id).name.map(|id| ast.lookup(id))
    }

    pub fn variable_definitions(&self) -> impl Iterator<Item = VariableDefinition<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .variable_definitions
            .iter()
            .map(|id| self.0.ast.read(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.ast.read(id))
    }

    pub fn selection_set(&self) -> impl Iterator<Item = Selection<'a>> {
        self.0
            .ast
            .lookup(self.0.id)
            .selection_set
            .iter()
            .map(|id| self.0.ast.read(id))
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
