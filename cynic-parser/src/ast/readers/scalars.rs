use crate::ast::ids::{AstLookup, ScalarDefinitionId};

use super::{directives::Directive, AstId, ReadContext};

pub struct ScalarDefinition<'a>(ReadContext<'a, ScalarDefinitionId>);

impl<'a> ScalarDefinition<'a> {
    pub fn name(&self) -> &str {
        let ast = &self.0.ast;

        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&str> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn directives(&self) -> impl Iterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(*id))
    }
}

impl AstId for ScalarDefinitionId {
    type Reader<'a> = ScalarDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ScalarDefinitionId>> for ScalarDefinition<'a> {
    fn from(value: ReadContext<'a, ScalarDefinitionId>) -> Self {
        Self(value)
    }
}
