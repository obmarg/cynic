use crate::{type_system::ids::InputObjectDefinitionId, AstLookup};

use super::{
    directives::Directive, input_values::InputValueDefinition, ReadContext, StringLiteral,
    TypeSystemId,
};

#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a>(ReadContext<'a, InputObjectDefinitionId>);

impl<'a> InputObjectDefinition<'a> {
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

    pub fn fields(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        self.0
            .document
            .lookup(self.0.id)
            .fields
            .iter()
            .map(|id| self.0.document.read(id))
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

impl TypeSystemId for InputObjectDefinitionId {
    type Reader<'a> = InputObjectDefinition<'a>;
}

impl<'a> From<ReadContext<'a, InputObjectDefinitionId>> for InputObjectDefinition<'a> {
    fn from(value: ReadContext<'a, InputObjectDefinitionId>) -> Self {
        Self(value)
    }
}
