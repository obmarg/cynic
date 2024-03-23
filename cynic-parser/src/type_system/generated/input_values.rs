#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, InputValueDefinitionId, StringLiteralId, TypeId, ValueId},
    types::Type,
    value::Value,
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct InputValueDefinitionRecord {
    pub name: StringId,
    pub ty: Option<TypeId>,
    pub description: Option<StringLiteralId>,
    pub default: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct InputValueDefinition<'a>(ReadContext<'a, InputValueDefinitionId>);

impl<'a> InputValueDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Option<Type<'a>> {
        let document = self.0.document;
        document.lookup(self.0.id).ty.map(|id| document.read(id))
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn default(&self) -> Option<Value<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .default
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
    pub fn span(&self) -> Option<Span> {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeSystemId for InputValueDefinitionId {
    type Reader<'a> = InputValueDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InputValueDefinitionId>> for InputValueDefinition<'a> {
    fn from(value: ReadContext<'a, InputValueDefinitionId>) -> Self {
        Self(value)
    }
}