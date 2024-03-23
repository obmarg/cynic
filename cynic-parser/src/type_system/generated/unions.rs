#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, StringLiteralId, UnionDefinitionId},
    ReadContext,
    StringLiteral::StringLiteral,
    TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    AstLookup,
};

pub struct UnionDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub members: IdRange<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Option<Span>,
}

#[derive(Clone, Copy)]
pub struct UnionDefinition<'a>(ReadContext<'a, UnionDefinitionId>);

impl<'a> UnionDefinition<'a> {
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
    pub fn members(&self) -> impl ExactSizeIterator<Item = &'a str> {
        let document = self.0.document;
        document
            .lookup(self.0.id)
            .members
            .map(|id| document.lookup(id))
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

impl TypeSystemId for UnionDefinitionId {
    type Reader<'a> = UnionDefinition<'a>;
}

impl<'a> From<ReadContext<'a, UnionDefinitionId>> for UnionDefinition<'a> {
    fn from(value: ReadContext<'a, UnionDefinitionId>) -> Self {
        Self(value)
    }
}