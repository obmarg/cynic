#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, FieldDefinitionId, InputValueDefinitionId, StringLiteralId, TypeId},
    input_values::InputValueDefinition,
    strings::StringLiteral,
    types::Type,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct FieldDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct FieldDefinition<'a>(ReadContext<'a, FieldDefinitionId>);

impl<'a> FieldDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let document = &self.0.document;
        document.lookup(document.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Type<'a> {
        let document = self.0.document;
        document.read(document.lookup(self.0.id).ty)
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| document.read(id))
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
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

impl TypeSystemId for FieldDefinitionId {
    type Reader<'a> = FieldDefinition<'a>;
}

impl<'a> From<ReadContext<'a, FieldDefinitionId>> for FieldDefinition<'a> {
    fn from(value: ReadContext<'a, FieldDefinitionId>) -> Self {
        Self(value)
    }
}
