use crate::{type_system::ids::ScalarDefinitionId, AstLookup};

use super::{directives::Directive, ReadContext, TypeSystemId};

#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a>(ReadContext<'a, ScalarDefinitionId>);

impl<'a> ScalarDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.document;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .directives
            .iter()
            .map(|id| ast.read(id))
    }
}

impl TypeSystemId for ScalarDefinitionId {
    type Reader<'a> = ScalarDefinition<'a>;
}

impl<'a> From<ReadContext<'a, ScalarDefinitionId>> for ScalarDefinition<'a> {
    fn from(value: ReadContext<'a, ScalarDefinitionId>) -> Self {
        Self(value)
    }
}
