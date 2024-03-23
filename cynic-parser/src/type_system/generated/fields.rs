#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, FieldDefinitionId, InputValueDefinitionId, StringLiteralId, TypeId},
    input_values::InputValueDefinition,
    types::Type,
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct FieldDefinitionRecord {
    pub name: StringId,
    pub ty: Option<TypeId>,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct FieldDefinition<'a>(ReadContext<'a, FieldDefinitionId>);

impl<'a> FieldDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn ty(&self) -> Option<Type<'a>> {
        let document = self.0.document;
        document.lookup(self.0.id).ty.map(|id| document.read(id))
    }
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> {
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

impl TypeSystemId for FieldDefinitionId {
    type Reader<'a> = FieldDefinition<'a>;
}

impl<'a> From<ReadContext<'a, FieldDefinitionId>> for FieldDefinition<'a> {
    fn from(value: ReadContext<'a, FieldDefinitionId>) -> Self {
        Self(value)
    }
}