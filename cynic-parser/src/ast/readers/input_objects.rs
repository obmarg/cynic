use crate::ast::{ids::InputObjectDefinitionId, AstLookup};

use super::{directives::Directive, input_values::InputValueDefinition, AstId, ReadContext};

#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a>(ReadContext<'a, InputObjectDefinitionId>);

impl<'a> InputObjectDefinition<'a> {
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

    pub fn fields(&self) -> impl Iterator<Item = InputValueDefinition<'a>> + 'a {
        self.0
            .ast
            .lookup(self.0.id)
            .fields
            .iter()
            .map(|id| self.0.ast.read(*id))
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

impl AstId for InputObjectDefinitionId {
    type Reader<'a> = InputObjectDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InputObjectDefinitionId>> for InputObjectDefinition<'a> {
    fn from(value: ReadContext<'a, InputObjectDefinitionId>) -> Self {
        Self(value)
    }
}
