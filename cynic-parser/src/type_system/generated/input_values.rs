#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, InputValueDefinitionId, StringLiteralId, TypeId, ValueId},
    strings::StringLiteral,
    types::Type,
    value::Value,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct InputValueDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<StringLiteralId>,
    pub default_value: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct InputValueDefinition<'a>(ReadContext<'a, InputValueDefinitionId>);

impl<'a> InputValueDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Type<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn default_value(&self) -> Option<Value<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .default_value
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
    pub fn span(&self) -> Span {
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
