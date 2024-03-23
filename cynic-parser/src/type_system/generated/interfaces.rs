#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    fields::FieldDefinition,
    ids::{DirectiveId, FieldDefinitionId, InterfaceDefinitionId, StringLiteralId},
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct InterfaceDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements: IdRange<StringId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct InterfaceDefinition<'a>(ReadContext<'a, InterfaceDefinitionId>);

impl<'a> InterfaceDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<StringLiteral<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .description
            .map(|id| document.read(id))
    }
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .fields
            .iter()
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
    pub fn implements(&self) -> impl ExactSizeIterator<Item = &'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .implements
            .map(|id| document.lookup(id))
    }
    pub fn span(&self) -> Option<Span> {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeSystemId for InterfaceDefinitionId {
    type Reader<'a> = InterfaceDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InterfaceDefinitionId>> for InterfaceDefinition<'a> {
    fn from(value: ReadContext<'a, InterfaceDefinitionId>) -> Self {
        Self(value)
    }
}