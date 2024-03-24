#[allow(unused_imports)]
use super::ids::StringId;
use super::{
    directives::Directive,
    ids::{DirectiveId, StringLiteralId, UnionDefinitionId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use crate::{
    common::{IdRange, OperationType},
    type_system::DirectiveLocation,
    AstLookup, Span,
};

pub struct UnionDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub members: Vec<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct UnionDefinition<'a>(ReadContext<'a, UnionDefinitionId>);

impl<'a> UnionDefinition<'a> {
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
    pub fn members(&self) -> impl ExactSizeIterator<Item = &'a str> + 'a {
        let document = &self.0.document;
        document
            .lookup(self.0.id)
            .members
            .iter()
            .map(|id| document.lookup(*id))
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

impl TypeSystemId for UnionDefinitionId {
    type Reader<'a> = UnionDefinition<'a>;
}

impl<'a> From<ReadContext<'a, UnionDefinitionId>> for UnionDefinition<'a> {
    fn from(value: ReadContext<'a, UnionDefinitionId>) -> Self {
        Self(value)
    }
}
