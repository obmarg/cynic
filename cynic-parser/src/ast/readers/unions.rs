use crate::ast::{ids::UnionDefinitionId, AstLookup};

use super::{directives::Directive, AstId, ReadContext};

pub struct UnionDefinition<'a>(ReadContext<'a, UnionDefinitionId>);

impl<'a> UnionDefinition<'a> {
    pub fn name(&self) -> &str {
        self.0.ast.lookup(self.0.ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<&str> {
        self.0
            .ast
            .lookup(self.0.id)
            .description
            .map(|id| self.0.ast.lookup(id))
    }

    pub fn members(&self) -> impl Iterator<Item = &'a str> + 'a {
        self.0
            .ast
            .lookup(self.0.id)
            .members
            .iter()
            .map(|id| self.0.ast.lookup(*id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        self.0
            .ast
            .lookup(self.0.id)
            .directives
            .iter()
            .map(|id| self.0.ast.read(*id))
    }
}

impl AstId for UnionDefinitionId {
    type Reader<'a> = UnionDefinition<'a>;
}

impl<'a> From<ReadContext<'a, UnionDefinitionId>> for UnionDefinition<'a> {
    fn from(value: ReadContext<'a, UnionDefinitionId>) -> Self {
        Self(value)
    }
}
