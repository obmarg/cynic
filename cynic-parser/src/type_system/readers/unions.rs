use crate::{type_system::ids::UnionDefinitionId, AstLookup};

use super::{directives::Directive, ReadContext, StringLiteral, TypeSystemId};

#[derive(Clone, Copy)]
pub struct UnionDefinition<'a>(ReadContext<'a, UnionDefinitionId>);

impl<'a> UnionDefinition<'a> {
    pub fn name(&self) -> &'a str {
        self.0
            .document
            .lookup(self.0.document.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<StringLiteral<'a>> {
        self.0
            .document
            .lookup(self.0.id)
            .description
            .map(|id| self.0.document.read(id))
    }

    pub fn members(&self) -> impl ExactSizeIterator<Item = &'a str> + 'a {
        self.0
            .document
            .lookup(self.0.id)
            .members
            .iter()
            .map(|id| self.0.document.lookup(*id))
    }

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0
            .document
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.document.read(id))
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
