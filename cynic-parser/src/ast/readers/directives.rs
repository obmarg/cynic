use crate::ast::{
    ids::{AstLookup, DirectiveDefinitionId, DirectiveId},
    DirectiveLocation,
};

use super::{arguments::Argument, input_values::InputValueDefinition, AstId, AstReader};

pub struct DirectiveDefinition<'a>(AstReader<'a, DirectiveDefinitionId>);

impl<'a> DirectiveDefinition<'a> {
    pub fn name(&self) -> &str {
        let ast = &self.0.ast;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn description(&self) -> Option<&'a str> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id).description.map(|id| ast.lookup(id))
    }

    pub fn arguments(&self) -> impl Iterator<Item = InputValueDefinition<'a>> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(*id))
    }

    pub fn is_repeatable(&self) -> bool {
        self.0.ast.lookup(self.0.id).repeatable
    }

    pub fn locations(&self) -> impl Iterator<Item = DirectiveLocation> + 'a {
        self.0.ast.lookup(self.0.id).locations.iter().copied()
    }
}

pub struct Directive<'a>(AstReader<'a, DirectiveId>);

impl<'a> Directive<'a> {
    pub fn name(&self) -> &str {
        let ast = &self.0.ast;

        ast.lookup(ast.lookup(self.0.id).name)
    }

    pub fn arguments(&self) -> impl Iterator<Item = Argument<'a>> {
        let ast = &self.0.ast;

        ast.lookup(self.0.id)
            .arguments
            .iter()
            .map(|id| ast.read(*id))
    }
}

impl AstId for DirectiveDefinitionId {
    type Reader<'a> = DirectiveDefinition<'a>;
}

impl<'a> From<AstReader<'a, DirectiveDefinitionId>> for DirectiveDefinition<'a> {
    fn from(value: AstReader<'a, DirectiveDefinitionId>) -> Self {
        Self(value)
    }
}

impl AstId for DirectiveId {
    type Reader<'a> = Directive<'a>;
}

impl<'a> From<AstReader<'a, DirectiveId>> for Directive<'a> {
    fn from(value: AstReader<'a, DirectiveId>) -> Self {
        Self(value)
    }
}
