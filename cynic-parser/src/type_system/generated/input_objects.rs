#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, InputObjectDefinitionId, InputValueDefinitionId, StringLiteralId},
    input_values::InputValueDefinition,
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct InputObjectDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<InputValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a>(ReadContext<'a, InputObjectDefinitionId>);

impl<'a> InputObjectDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn fields(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .fields
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
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeSystemId for InputObjectDefinitionId {
    type Reader<'a> = InputObjectDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InputObjectDefinitionId>> for InputObjectDefinition<'a> {
    fn from(value: ReadContext<'a, InputObjectDefinitionId>) -> Self {
        Self(value)
    }
}
