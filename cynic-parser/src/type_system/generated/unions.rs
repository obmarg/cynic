use super::prelude::*;
use super::{
    directives::Directive,
    ids::{DirectiveId, StringLiteralId, UnionDefinitionId},
    strings::StringLiteral,
    ReadContext, TypeSystemId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub struct UnionDefinitionRecord {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub members: Vec<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub struct UnionDefinition<'a>(pub(in super::super) ReadContext<'a, UnionDefinitionId>);

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
    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        let document = self.0.document;
        super::Iter::new(document.lookup(self.0.id).directives, document)
    }
    pub fn span(&self) -> Span {
        let document = self.0.document;
        document.lookup(self.0.id).span
    }
}

impl UnionDefinition<'_> {
    pub fn id(&self) -> UnionDefinitionId {
        self.0.id
    }
}

impl fmt::Debug for UnionDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("members", &self.members().collect::<Vec<_>>())
            .field("directives", &self.directives())
            .field("span", &self.span())
            .finish()
    }
}

impl TypeSystemId for UnionDefinitionId {
    type Reader<'a> = UnionDefinition<'a>;
}

impl IdReader for UnionDefinition<'_> {
    type Id = UnionDefinitionId;
}

impl<'a> From<ReadContext<'a, UnionDefinitionId>> for UnionDefinition<'a> {
    fn from(value: ReadContext<'a, UnionDefinitionId>) -> Self {
        Self(value)
    }
}
