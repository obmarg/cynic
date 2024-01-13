use crate::ast::ids::{AstLookup, UnionDefinitionId};

use super::{directives::Directive, AstId, AstReader};

pub struct UnionDefinition<'a>(AstReader<'a, UnionDefinitionId>);

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

impl<'a> From<AstReader<'a, UnionDefinitionId>> for UnionDefinition<'a> {
    fn from(value: AstReader<'a, UnionDefinitionId>) -> Self {
        Self(value)
    }
}
