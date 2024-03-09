use crate::type_system::{
    ids::{DirectiveDefinitionId, DirectiveId},
    DirectiveLocation,
};
use crate::AstLookup;

use super::{arguments::Argument, input_values::InputValueDefinition, ReadContext, TypeSystemId};

#[derive(Clone, Copy)]
pub struct DirectiveDefinition<'a>(ReadContext<'a, DirectiveDefinitionId>);

impl<'a> DirectiveDefinition<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.document;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(id))
    }

    pub fn is_repeatable(&self) -> bool {
        self.0.document.lookup(self.0.id).repeatable
    }

    pub fn locations(&self) -> impl ExactSizeIterator<Item = DirectiveLocation> + 'a {
        self.0.document.lookup(self.0.id).locations.iter().copied()
    }
}

#[derive(Clone, Copy)]
pub struct Directive<'a>(ReadContext<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> &'a str {
        let ast = &self.0.document;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn arguments(&self) -> impl ExactSizeIterator<Item = Argument<'a>> {
        let ast = &self.0.document;

        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(*id))
    }
}

impl TypeSystemId for DirectiveDefinitionId {
    type Reader<'a> = DirectiveDefinition<'a>;
}

impl<'a> From<ReadContext<'a, DirectiveDefinitionId>> for DirectiveDefinition<'a> {
    fn from(value: ReadContext<'a, DirectiveDefinitionId>) -> Self {
        Self(value)
    }
}

impl TypeSystemId for DirectiveId {
    type Reader<'a> = Directive<'a>;
}

impl<'a> From<ReadContext<'a, DirectiveId>> for Directive<'a> {
    fn from(value: ReadContext<'a, DirectiveId>) -> Self {
        Self(value)
    }
}
