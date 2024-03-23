#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directive::Directive,
    ids::{DirectiveId, TypeId, ValueId, VariableDefinitionId},
    types::Type,
    value::Value,
    ExecutableId, ReadContext,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
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
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn default_value(&self) -> Option<Value<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .default_value
            .map(|id| document.read(id))
    }
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| document.read(id))
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

