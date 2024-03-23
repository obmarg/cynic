#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, EnumDefinitionId, EnumValueDefinitionId, StringLiteralId},
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct EnumDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub values: IdRange<EnumValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct EnumDefinition<'a>(ReadContext<'a, EnumDefinitionId>);

impl<'a> EnumDefinition<'a> {
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
    pub fn values(&self) -> impl ExactSizeIterator<Item = EnumValueDefinition<'a>> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .values
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
    pub fn span(&self) -> Option<Span> {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl TypeSystemId for EnumDefinitionId {
    type Reader<'a> = EnumDefinition<'a>;
}

impl<'a> From<ReadContext<'a, EnumDefinitionId>> for EnumDefinition<'a> {
    fn from(value: ReadContext<'a, EnumDefinitionId>) -> Self {
        Self(value)
    }
}

pub struct EnumValueDefinitionRecord {
    pub value: StringId,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct EnumValueDefinition<'a>(ReadContext<'a, EnumValueDefinitionId>);

impl<'a> EnumValueDefinition<'a> {
    pub fn value(&self) -> &'a str {
        let ast = &self.0.document;
        ast.lookup(ast.lookup(self.0.id).value)
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

impl TypeSystemId for EnumValueDefinitionId {
    type Reader<'a> = EnumValueDefinition<'a>;
}

impl<'a> From<ReadContext<'a, EnumValueDefinitionId>> for EnumValueDefinition<'a> {
    fn from(value: ReadContext<'a, EnumValueDefinitionId>) -> Self {
        Self(value)
    }
}