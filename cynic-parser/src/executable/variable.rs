use crate::{common::IdRange, AstLookup};

use super::{
    directive::Directive,
    ids::{DirectiveId, StringId, TypeId, ValueId, VariableDefinitionId},
    types::Type,
    value::Value,
    ExecutableId, ReadContext,
};

pub struct VariableDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub default_value: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
}

#[derive(Clone, Copy)]
pub struct VariableDefinition<'a>(ReadContext<'a, VariableDefinitionId>);

impl<'a> VariableDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn ty(&self) -> Type<'a> {
        let ast = &self.0.document;
        ast.read(ast.lookup(self.0.id).ty)
    }

    pub fn default_value(&self) -> Option<Value<'a>> {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .default_value
            .map(|id| self.0.document.read(id))
    }

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0
            .document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.document.read(id))
    }
}

impl ExecutableId for VariableDefinitionId {
    type Reader<'a> = VariableDefinition<'a>;
}

impl<'a> From<ReadContext<'a, VariableDefinitionId>> for VariableDefinition<'a> {
    fn from(value: ReadContext<'a, VariableDefinitionId>) -> Self {
        Self(value)
    }
}
